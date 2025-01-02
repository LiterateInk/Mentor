pub fn remove_from_until(
  content: &str,
  start_str: &str,
  end_str: &str,
  remove_end: bool,
) -> String {
  let start = content.find(start_str).expect("start_str not found");
  let mut end = start + content[start..].find(end_str).expect("end_str not found");

  if remove_end {
    end += end_str.len();
  }

  format!("{}{}", &content[..start], &content[end..])
}
