// Typing lecture: command pattern.

trait Command {
    fn execute(&self, current: f64) -> f64;

    fn undo(&self, before: f64, _current: f64) -> f64 {
        before
    }
}

struct AddCmd(f64);
struct MulCmd(f64);
struct SetCmd(f64);

impl Command for AddCmd {
    fn execute(&self, current: f64) -> f64 {
        current + self.0
    }
}

impl Command for MulCmd {
    fn execute(&self, current: f64) -> f64 {
        current * self.0
    }
}

impl Command for SetCmd {
    fn execute(&self, _current: f64) -> f64 {
        self.0
    }
}

struct HistoryEntry {
    command: Box<dyn Command>,
    before: f64,
}

#[derive(Default)]
struct Calculator {
    value: f64,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

impl Calculator {
    fn value(&self) -> f64 {
        self.value
    }

    fn execute(&mut self, command: Box<dyn Command>) {
        let before = self.value;
        self.value = command.execute(self.value);
        self.undo_stack.push(HistoryEntry { command, before });
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };

        self.value = entry.command.undo(entry.before, self.value);
        self.redo_stack.push(entry);
        true
    }

    fn redo(&mut self) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };

        let before = self.value;
        self.value = entry.command.execute(self.value);
        self.undo_stack.push(HistoryEntry {
            command: entry.command,
            before,
        });
        true
    }
}

fn main() {
    let mut calculator = Calculator::default();
    calculator.execute(Box::new(SetCmd(10.0)));
    println!("{}", calculator.value());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_commands() {
        let mut calculator = Calculator::default();

        calculator.execute(Box::new(SetCmd(10.0)));
        calculator.execute(Box::new(AddCmd(5.0)));
        calculator.execute(Box::new(MulCmd(3.0)));

        assert_eq!(calculator.value(), 45.0);
    }

    #[test]
    fn supports_undo_and_redo() {
        let mut calculator = Calculator::default();

        calculator.execute(Box::new(SetCmd(10.0)));
        calculator.execute(Box::new(AddCmd(5.0)));
        calculator.execute(Box::new(MulCmd(3.0)));

        assert!(calculator.undo());
        assert_eq!(calculator.value(), 15.0);
        assert!(calculator.undo());
        assert_eq!(calculator.value(), 10.0);
        assert!(calculator.redo());
        assert_eq!(calculator.value(), 15.0);
    }

    #[test]
    fn new_command_clears_redo_history() {
        let mut calculator = Calculator::default();

        calculator.execute(Box::new(SetCmd(2.0)));
        calculator.execute(Box::new(AddCmd(3.0)));
        assert!(calculator.undo());
        calculator.execute(Box::new(MulCmd(10.0)));

        assert!(!calculator.redo());
        assert_eq!(calculator.value(), 20.0);
    }
}
