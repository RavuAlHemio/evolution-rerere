pub trait StringExt {
    fn split_last<'s>(&'s self, c: char) -> Option<(&'s str, &'s str)>;
}
impl StringExt for str {
    fn split_last<'s>(&'s self, c: char) -> Option<(&'s str, &'s str)> {
        let last_c_index = self.rfind(c)?;
        let c_len = c.len_utf8();
        let front = &self[0..last_c_index];
        let back = &self[last_c_index+c_len..];
        Some((front, back))
    }
}
