use regex::Regex;

/// Teams のミーティングリンクを抽出する関数
///
/// # 引数
///
/// * `description` - Teams リンクを含む可能性のある文字列
///
/// # 戻り値
///
/// Teams リンクが見つかった場合は `Some(String)`, 見つからなかった場合は `None`
///
/// # Examples
///
/// ```
/// use calendar_notice::google_calendar::extract_teams_link;
///
/// let description = "ミーティングリンク: https://teams.microsoft.com/l/meetup-join/19%3ameeting_ABC123...";
/// assert_eq!(
///     extract_teams_link(description),
///     Some("https://teams.microsoft.com/l/meetup-join/19%3ameeting_ABC123...".to_string())
/// );
///
/// let no_link = "この文字列にはTeamsリンクが含まれていません。";
/// assert_eq!(extract_teams_link(no_link), None);
/// ```
pub fn extract_teams_link(description: &str) -> Option<String> {
    let re = Regex::new(r"https://teams\.microsoft\.com/l/meetup-join/[^\s]+").unwrap();
    re.find(description).map(|m| m.as_str().to_string())
}
