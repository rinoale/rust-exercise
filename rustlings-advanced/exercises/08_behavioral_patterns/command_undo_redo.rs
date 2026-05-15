// Typing lecture: command pattern.
//
// Commands package operations as values. The calculator stores commands with
// the previous value so undo and redo can move through history.
//
// TODO: Implement commands, undo, and redo.

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
        todo!("add the command value");
    }
}

impl Command for MulCmd {
    fn execute(&self, current: f64) -> f64 {
        todo!("multiply by the command value");
    }
}

impl Command for SetCmd {
    fn execute(&self, _current: f64) -> f64 {
        todo!("replace the current value");
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
        todo!("execute a command and push it onto undo history");
    }

    fn undo(&mut self) -> bool {
        todo!("restore the previous value if possible");
    }

    fn redo(&mut self) -> bool {
        todo!("re-execute the most recently undone command if possible");
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
