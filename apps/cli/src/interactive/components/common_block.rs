use ratatui::{
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::Block,
};

type Instruction = (&'static str, &'static str);

pub struct CommonBlock {
    title: String,
    instructions: Vec<Instruction>,
    border_color: Color,
}

impl CommonBlock {
    pub fn new(title: String) -> Self {
        Self {
            title,
            instructions: vec![],
            border_color: Color::Gray,
        }
    }

    pub fn add_instruction(mut self, ins: Instruction) -> Self {
        self.instructions.push(ins);
        self
    }

    pub fn set_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
}

impl From<CommonBlock> for Block<'_> {
    fn from(value: CommonBlock) -> Self {
        let mut instructions: Vec<Span<'_>> = value
            .instructions
            .into_iter()
            .flat_map(|(key, value)| {
                vec![
                    format!(" {} ", key).into(),
                    format!("<{}>", value).magenta().bold(),
                ]
            })
            .collect();

        instructions.push(" ".into());

        let instructions = Line::from(instructions).centered();
        let title = Line::from(format!(" {} ", value.title.bold())).left_aligned();

        Block::bordered()
            .border_style(value.border_color)
            .title(title)
            .title_bottom(instructions)
            .border_set(border::ROUNDED)
    }
}
