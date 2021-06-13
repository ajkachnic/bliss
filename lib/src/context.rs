pub trait Context {
    fn context<T: ToString>(&self, context: T) -> Self;
    fn get_context(&self) -> Vec<String>;

    fn display_context(&self) -> String {
        let items: Vec<String> = self
            .get_context()
            .iter()
            .enumerate()
            .map(|(i, context)| format!("  {}: {}", i + 1, context))
            .collect();
        if items.is_empty() {
            return String::new();
        }
        format!("Context:\n{}", items.join("\n"))
    }
}

pub trait Hint {
    fn hint<T: ToString>(&self, hint: T) -> Self;
}
