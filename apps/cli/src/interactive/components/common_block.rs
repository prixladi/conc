use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Span},
    widgets::Block,
};

type Instruction = (&'static str, &'static str);

pub struct CommonBlock {
    title: String,
    instructions: Vec<Instruction>,
}

impl CommonBlock {
    pub fn new(title: String) -> Self {
        Self {
            title,
            instructions: vec![],
        }
    }

    pub fn add_instruction(mut self, ins: Instruction) -> Self {
        self.instructions.push(ins);
        self
    }
}

impl<'a> From<CommonBlock> for Block<'a> {
    fn from(value: CommonBlock) -> Self {
        let mut instructions: Vec<Span<'_>> = value
            .instructions
            .into_iter()
            .flat_map(|(key, value)| {
                vec![
                    format!(" {} ", key).into(),
                    format!("<{}>", value).blue().bold(),
                ]
            })
            .collect();

        instructions.push(" ".into());

        let instructions = Line::from(instructions).centered();
        let title = Line::from(format!(" {} ", value.title.bold())).left_aligned();

        Block::bordered()
            .title(title)
            .title_bottom(instructions)
            .border_set(border::THICK)
    }
}
