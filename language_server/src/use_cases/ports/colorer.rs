use crate::entities::{Color, ColorType, Colors, Column, Match, RowMetadata};

pub trait Colorer {
    fn color_text(&self, text: &str, color_type: ColorType) -> Option<Color>;

    fn update_palette(&mut self, text: String, colors: Colors);

    fn color_intervals(
        &self,
        row_matches: &[Match],
        row_meta: &RowMetadata,
    ) -> Vec<(Column, Column)> {
        let mut color_intervals = vec![];

        color_intervals.extend(
            row_matches
                .iter()
                .zip(row_matches.iter().skip(1))
                .map(|(match1, match2)| (*match1.column(), *match2.column())),
        );

        let last_match = row_matches
                .last()
                .expect("This is always called when there is a match in user's text so there is something in the slice");
        let line_end = (row_meta.line_len() - 1).into();
        color_intervals.push((*last_match.column(), line_end));

        // Extend the first match interval to include the text up until the
        // first non-zero column
        color_intervals[0].0 = *row_meta.first_non_zero_pos();
        color_intervals
    }
}
