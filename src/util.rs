pub mod http;
pub mod inflector;
pub mod jpeg;

pub fn format_duration(t: i32) -> String {
    let minutes = t / 60;
    let seconds = t % 60;
    format!("{}:{:02}", minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(8), "0:08");
        assert_eq!(format_duration(32), "0:32");
        assert_eq!(format_duration(63), "1:03");
        assert_eq!(format_duration(207), "3:27");
        assert_eq!(format_duration(671), "11:11");
    }
}
