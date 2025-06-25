use std::sync::LazyLock;
use regex::Regex;

#[derive(Debug)]
pub struct IssueIds {
    pub issue_id: u64,
    pub comment_id: Option<u64>
}

static COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://github\.com/rh-hideout/pokeemerald-expansion/issues/(?<issue_id>\d+)\#issuecomment-(?<comment_id>\d+)").unwrap());
static ISSUE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://github\.com/rh-hideout/pokeemerald-expansion/(issues|pull)/(?<issue_id>\d+)").unwrap());

impl IssueIds {
    pub fn from_url(url: impl Into<String>) -> Option<Self> {
        let str_url = &url.into();
        if let Some(caps) = COMMENT_RE.captures(str_url) {
            return Some(Self {
                issue_id: caps["issue_id"].parse().unwrap(),
                comment_id: Some(caps["comment_id"].parse().unwrap())
            });
        } else if let Some(caps) = ISSUE_RE.captures(str_url) {
            return Some(Self {
                issue_id: caps["issue_id"].parse().unwrap(),
                comment_id: None
            });
        } else {
            return None
        }
    }
}
