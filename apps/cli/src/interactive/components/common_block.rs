use ratatui::{
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::Block,
};

type Instruction = (&'static str, &'static str);

pub struct CommonBlock {
    title: String,
    top_info: Option<String>,
    instructions: Vec<Instruction>,
    border_color: Color,
}

impl CommonBlock {
    pub fn new(title: String) -> Self {
        Self {
            title,
            instructions: vec![],
            border_color: Color::Gray,
            top_info: None,
        }
    }

    pub fn add_top_info(mut self, info: String) -> Self {
        self.top_info = Some(info);
        self
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

        if !instructions.is_empty() {
            instructions.push(" ".into());
        }

        let instructions = Line::from(instructions).centered();
        let title = Line::from(format!(" {} ", value.title.bold())).left_aligned();

        let mut block = Block::bordered()
            .border_style(value.border_color)
            .title(title)
            .title_bottom(instructions)
            .border_set(border::ROUNDED);

        if let Some(top_info) = value.top_info {
            block = block.title(Line::from(format!(" {} ", top_info)).right_aligned());
        }

        block
    }
}
