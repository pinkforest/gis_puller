//! Module au implements acquisition of Geoscape datasets thru data.gov.au provider using RESTful protocol

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
struct Distribution {
  identifier: String,
  downloadURL: String,
  modified: Option<String>,
  mediaType: Option<String>
}

impl Distribution {
  /// fetcher_matcher regex match for downloadURL
  fn am_i_right(self: &Self, r: &Regex) -> bool {
    return r.is_match(&self.downloadURL);
  }
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
struct DataSets {
  distributions: Vec<Distribution>
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
struct SearchDSResult {
  dataSets: Vec<DataSets>
}

use regex::Regex;

macro_rules! matcher_regex {
    ($re:expr $(,)?) => {{
      static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
      RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

use crate::settings::MatcherSettingsDerive;

quick_error! {
  #[derive(Debug)]
  /// FetcherMatcherError combines all the errors returned from fetcher_matcher
  pub enum FetcherMatcherError {
    /// All API Errors from reqwest
    API(err: reqwest::Error) {
      display("{}", err)
      from()
    }
    /// All IO errors
    Io(err: std::io::Error) {
      source(err)
      from()
    }
  }
}

/// fetcher_matcher is a prototype for generic interface implementing simple data acquisition mechanism
pub async fn fetcher_matcher(settings: &MatcherSettingsDerive) -> Result<Vec<String>, FetcherMatcherError> {

  // @TODO: Create error type let's not panic because people mess configuration? Library should not panic.
  let set_matcher = settings.matcher.as_ref().expect("matcher must be set");
  let set_url = settings.rest_url.as_ref().expect("rest_url must be set");
  let set_query = settings.query.as_ref().expect("query must be set");
  
  let re_valid_filename = matcher_regex!(&set_matcher);

  let client = reqwest::Client::new();
  
  let response = client.get(set_url)
    .query(&[("query", set_query)])
    .send().await?;

  let search_result: SearchDSResult = response.json().await?;
  
  let matches: Vec<_> = search_result
    .dataSets
    .iter()
    .flat_map(|ds| {
      ds.distributions
        .iter()
        .filter(|dist| dist.am_i_right(&re_valid_filename))
        .map(|dist| dist.downloadURL.clone())
    })
    .collect(); 
   
  Ok(matches)
}
