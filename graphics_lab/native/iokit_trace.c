// Trace only this process's GPU connections. Calls and buffers are forwarded unchanged.
#include <IOKit/IOKitLib.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define CAPTURE_BYTES 128
#define CAPTURE_SCALARS 16
#define MAX_CONNECTIONS 256

static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_once_t once = PTHREAD_ONCE_INIT;
static FILE *log_file;
static io_connect_t gpu_connections[MAX_CONNECTIONS];
static size_t connection_count;
static uint64_t sequence;
static _Thread_local unsigned call_depth;

static void init_log(void) {
    const char *path = getenv("GRAPHICS_LAB_TRACE");
    if (!path) return;
    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (fd >= 0) {
        log_file = fdopen(fd, "w");
        if (!log_file) close(fd);
    }
}

static bool is_gpu(io_connect_t connection) {
    bool found = false;
    pthread_mutex_lock(&lock);
    for (size_t i = 0; i < connection_count; ++i)
        if (gpu_connections[i] == connection) { found = true; break; }
    pthread_mutex_unlock(&lock);
    return found;
}

static void prefix(const char *kind, io_connect_t connection, kern_return_t result) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    fprintf(log_file, "{\"sequence\":%llu,\"monotonic_ns\":%llu,\"kind\":\"%s\",\"connection\":%u,\"return_code\":%u",
        (unsigned long long)++sequence,
        (unsigned long long)now.tv_sec * 1000000000ULL + (unsigned long long)now.tv_nsec,
        kind, connection, (uint32_t)result);
}

static void hex(const void *data, size_t length) {
    const unsigned char *bytes = data;
    fputc('"', log_file);
    if (data) for (size_t i = 0; i < length; ++i) fprintf(log_file, "%02x", bytes[i]);
    fputc('"', log_file);
}

static void scalars(const uint64_t *data, uint32_t count) {
    fputc('[', log_file);
    if (data) for (uint32_t i = 0; i < count && i < CAPTURE_SCALARS; ++i)
        fprintf(log_file, "%s%llu", i ? "," : "", (unsigned long long)data[i]);
    fputc(']', log_file);
}

static void record_call(const char *kind, io_connect_t connection, uint32_t selector,
    kern_return_t result, const uint64_t *input, uint32_t input_count,
    const void *input_structure, size_t input_size, const uint64_t *output,
    uint32_t output_count, const void *output_structure, size_t output_size,
    size_t output_capacity) {
    pthread_once(&once, init_log);
    if (!log_file) return;
    pthread_mutex_lock(&lock);
    prefix(kind, connection, result);
    fprintf(log_file, ",\"selector\":%u,\"input_count\":%u,\"input_scalars\":", selector, input_count);
    scalars(input, input_count);
    fprintf(log_file, ",\"input_size\":%zu,\"input_hex\":", input_size);
    hex(input_structure, input_size < CAPTURE_BYTES ? input_size : CAPTURE_BYTES);
    fprintf(log_file, ",\"output_count\":%u,\"output_scalars\":", output_count);
    scalars(output, result == KERN_SUCCESS ? output_count : 0);
    fprintf(log_file, ",\"output_size\":%zu,\"output_hex\":", output_size);
    size_t safe_size = output_size < output_capacity ? output_size : output_capacity;
    // Variable-size output uses a pointer-to-pointer ABI. Do not interpret it as a buffer.
    if (output_capacity > 64 * 1024 * 1024 || result != KERN_SUCCESS) safe_size = 0;
    hex(output_structure, safe_size < CAPTURE_BYTES ? safe_size : CAPTURE_BYTES);
    fputs("}\n", log_file);
    fflush(log_file);
    pthread_mutex_unlock(&lock);
}

static kern_return_t traced_open(io_service_t service, task_port_t task, uint32_t type, io_connect_t *connection) {
    kern_return_t result = IOServiceOpen(service, task, type, connection);
    if (!IOObjectConformsTo(service, "IOGPU")) return result;
    pthread_once(&once, init_log);
    pthread_mutex_lock(&lock);
    io_connect_t port = result == KERN_SUCCESS && connection ? *connection : 0;
    if (port && connection_count < MAX_CONNECTIONS) gpu_connections[connection_count++] = port;
    if (log_file) {
        prefix("open", port, result);
        // Numeric identifiers avoid inserting unescaped driver-provided strings into JSON.
        uint64_t registry_id = 0;
        IORegistryEntryGetRegistryEntryID(service, &registry_id);
        fprintf(log_file, ",\"client_type\":%u,\"registry_id\":%llu}\n", type, (unsigned long long)registry_id);
        fflush(log_file);
    }
    pthread_mutex_unlock(&lock);
    return result;
}

static kern_return_t traced_close(io_connect_t connection) {
    bool tracked = is_gpu(connection);
    kern_return_t result = IOServiceClose(connection);
    if (tracked) {
        pthread_mutex_lock(&lock);
        if (log_file) { prefix("close", connection, result); fputs("}\n", log_file); fflush(log_file); }
        for (size_t i = 0; i < connection_count; ++i) {
            if (gpu_connections[i] == connection) { gpu_connections[i] = gpu_connections[--connection_count]; break; }
        }
        pthread_mutex_unlock(&lock);
    }
    return result;
}

static kern_return_t traced_method(mach_port_t connection, uint32_t selector,
    const uint64_t *input, uint32_t input_count, const void *input_structure,
    size_t input_size, uint64_t *output, uint32_t *output_count,
    void *output_structure, size_t *output_size) {
    size_t capacity = output_size ? *output_size : 0;
    uint32_t scalar_capacity = output_count ? *output_count : 0;
    bool outer = call_depth++ == 0;
    kern_return_t result = IOConnectCallMethod(connection, selector, input, input_count,
        input_structure, input_size, output, output_count, output_structure, output_size);
    --call_depth;
    if (outer && is_gpu(connection)) {
        uint32_t count = output_count && *output_count < scalar_capacity ? *output_count : scalar_capacity;
        record_call("method", connection, selector, result, input, input_count, input_structure,
            input_size, output, count, output_structure, output_size ? *output_size : 0, capacity);
    }
    return result;
}

static kern_return_t traced_struct(mach_port_t connection, uint32_t selector,
    const void *input, size_t input_size, void *output, size_t *output_size) {
    size_t capacity = output_size ? *output_size : 0;
    bool outer = call_depth++ == 0;
    kern_return_t result = IOConnectCallStructMethod(connection, selector, input, input_size, output, output_size);
    --call_depth;
    if (outer && is_gpu(connection)) record_call("struct_method", connection, selector, result,
        NULL, 0, input, input_size, NULL, 0, output, output_size ? *output_size : 0, capacity);
    return result;
}

static kern_return_t traced_scalar(mach_port_t connection, uint32_t selector,
    const uint64_t *input, uint32_t input_count, uint64_t *output, uint32_t *output_count) {
    uint32_t capacity = output_count ? *output_count : 0;
    bool outer = call_depth++ == 0;
    kern_return_t result = IOConnectCallScalarMethod(connection, selector, input, input_count, output, output_count);
    --call_depth;
    if (outer && is_gpu(connection)) {
        uint32_t count = output_count && *output_count < capacity ? *output_count : capacity;
        record_call("scalar_method", connection, selector, result, input, input_count, NULL, 0,
            output, count, NULL, 0, 0);
    }
    return result;
}

#define INTERPOSE(replacement, original) \
    __attribute__((used)) static struct { const void *new_function; const void *old_function; } \
    interpose_##original __attribute__((section("__DATA,__interpose"))) = \
        { (const void *)&replacement, (const void *)&original }

INTERPOSE(traced_open, IOServiceOpen);
INTERPOSE(traced_close, IOServiceClose);
INTERPOSE(traced_method, IOConnectCallMethod);
INTERPOSE(traced_struct, IOConnectCallStructMethod);
INTERPOSE(traced_scalar, IOConnectCallScalarMethod);
