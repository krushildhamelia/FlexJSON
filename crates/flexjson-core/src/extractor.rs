use crate::types::Fragment;

pub fn extract(input: &str) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    let mut current_line = 1;
    let mut current_col = 1;
    let mut last_idx = 0;

    let update_line_col = |pos: usize, line: &mut usize, col: &mut usize, last: &mut usize| {
        for j in *last..pos {
            if bytes[j] == b'\n' {
                *line += 1;
                *col = 1;
            } else if input.is_char_boundary(j) {
                *col += 1;
            }
        }
        *last = pos;
    };

    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'{' || ch == b'[' {
            update_line_col(i, &mut current_line, &mut current_col, &mut last_idx);
            let start = i;
            let start_line = current_line;
            let start_col = current_col;
            let fragment_type = if ch == b'{' { "object" } else { "array" };

            let mut depth = 0;
            let mut in_string = false;
            let mut quote_char = 0u8;

            while i < bytes.len() {
                let curr_ch = bytes[i];
                if !in_string {
                    if curr_ch == b'"' || curr_ch == b'\'' {
                        in_string = true;
                        quote_char = curr_ch;
                    } else if curr_ch == b'{' || curr_ch == b'[' {
                        depth += 1;
                    } else if curr_ch == b'}' || curr_ch == b']' {
                        depth -= 1;
                        if depth == 0 {
                            let end_offset = i + 1;
                            fragments.push(Fragment {
                                raw: input[start..end_offset].to_string(),
                                start_offset: start,
                                end_offset,
                                start_line,
                                start_col,
                                fragment_type: fragment_type.to_string(),
                            });
                            break;
                        }
                    }
                } else {
                    if curr_ch == quote_char {
                        let mut backslashes = 0;
                        let mut back_i = i.saturating_sub(1);
                        while back_i >= start && bytes[back_i] == b'\\' {
                            backslashes += 1;
                            if back_i == 0 { break; }
                            back_i -= 1;
                        }
                        if backslashes % 2 == 0 {
                            in_string = false;
                        }
                    }
                }
                i += 1;
            }
            if depth > 0 || in_string {
                fragments.push(Fragment {
                    raw: input[start..i].to_string(),
                    start_offset: start,
                    end_offset: i,
                    start_line,
                    start_col,
                    fragment_type: fragment_type.to_string(),
                });
            }
        } else {
            i += 1;
        }
    }

    fragments
}
