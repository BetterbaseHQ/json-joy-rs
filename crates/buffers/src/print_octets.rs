//! Debug utility for printing octets as hex strings.

/// Formats a byte slice as a hex string for debugging.
///
/// # Arguments
///
/// * `octets` - The byte slice to format.
/// * `max` - Maximum number of bytes to display (default: 16).
///
/// # Example
///
/// ```
/// use json_joy_buffers::print_octets;
///
/// assert_eq!(print_octets(&[0x01, 0x02, 0x0a, 0xff], 16), "01 02 0a ff");
/// assert_eq!(print_octets(&[], 16), "");
/// ```
pub fn print_octets(octets: &[u8], max: usize) -> String {
    if octets.is_empty() {
        return String::new();
    }

    let mut result = format!("{:02x}", octets[0]);
    for &byte in octets.iter().take(max).skip(1) {
        result.push_str(&format!(" {:02x}", byte));
    }

    if octets.len() > max {
        result.push_str(&format!("... ({} more)", octets.len() - max));
    }

    result
}

/// Formats a byte slice as a hex string with default max of 16 bytes.
///
/// # Example
///
/// ```
/// use json_joy_buffers::print_octets_default;
///
/// assert_eq!(print_octets_default(&[0x01, 0x02]), "01 02");
/// ```
pub fn print_octets_default(octets: &[u8]) -> String {
    print_octets(octets, 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_octets_empty() {
        assert_eq!(print_octets(&[], 16), "");
    }

    #[test]
    fn test_print_octets_single() {
        assert_eq!(print_octets(&[0x01], 16), "01");
    }

    #[test]
    fn test_print_octets_multiple() {
        assert_eq!(print_octets(&[0x01, 0x02, 0x0a, 0xff], 16), "01 02 0a ff");
    }

    #[test]
    fn test_print_octets_truncated() {
        let data: Vec<u8> = (0..20).collect();
        let result = print_octets(&data, 10);
        assert!(result.ends_with("... (10 more)"));
    }
}
