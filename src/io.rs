use std::io::{self, Write};

pub trait PromptLines {
    fn prompt_lines(self, prompt: &str) -> PromptLinesIter;
}

pub struct PromptLinesIter {
    lines: io::Lines<io::StdinLock<'static>>,
    prompt: String,
}

impl Iterator for PromptLinesIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        print!("{} ", self.prompt);
        io::stdout().flush().unwrap();
        self.lines
            .next()
            .map(|result| result.expect("Got an error for call to get next input line."))
    }
}

impl PromptLines for std::io::Stdin {
    fn prompt_lines(self, prompt: &str) -> PromptLinesIter {
        PromptLinesIter {
            lines: io::stdin().lines(),
            prompt: prompt.into(),
        }
    }
}
