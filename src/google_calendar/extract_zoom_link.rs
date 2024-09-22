use regex::Regex;

/// Zoom のミーティングリンクを抽出する関数
///
/// # 引数
///
/// * `description` - Zoom リンクを含む可能性のある文字列
///
/// # 戻り値
///
/// Zoom リンクが見つかった場合は `Some(String)`, 見つからなかった場合は `None`
///
/// # Examples
///
/// ```
/// use calendar_notice::google_calendar::extract_zoom_link;
///
/// let description = "ミーティングリンク: https://zoom.us/j/95428352872?pwd=SYUHanJf8xXQR51lhTKI8u3RSDWLVF.1";
/// assert_eq!(
///     extract_zoom_link(description),
///     Some("https://zoom.us/j/95428352872?pwd=SYUHanJf8xXQR51lhTKI8u3RSDWLVF.1".to_string())
/// );
///
/// let no_link = "この文字列にはZoomリンクが含まれていません。";
/// assert_eq!(extract_zoom_link(no_link), None);
///
/// let webinar_link = "ウェビナーに参加: https://us02.zoom.us/w/98765432100";
/// assert_eq!(
///     extract_zoom_link(webinar_link),
///     Some("https://zoom.us/j/98765432100".to_string())
/// );
/// ```
pub fn extract_zoom_link(description: &str) -> Option<String> {
    let re =
        Regex::new(r"https://(?:[\w-]+\.)?zoom\.us/(?:j|w|wc)/(\d+)(?:\?pwd=([a-zA-Z0-9.]+))?")
            .unwrap();
    re.captures(description).map(|cap| {
        let meeting_id = &cap[1];
        let password = cap.get(2).map(|m| m.as_str());

        match password {
            Some(pwd) => format!("https://zoom.us/j/{}?pwd={}", meeting_id, pwd),
            None => format!("https://zoom.us/j/{}", meeting_id),
        }
    })
}
