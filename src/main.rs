mod core;
mod adapters;

#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate ureq;
extern crate serde_json;





fn main() {
}




// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR&mode=artlist&maxrecords=250&format=json&startdatetime=20230617125228&enddatetime=20230617133000
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR(%22climate%20change%22%20OR%20%22global%20warming%22)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
//
// https://api.gdeltproject.org/api/v2/doc/doc?query=sourcelang:french sourcecountry:FR( climate change OR global warming)&mode=artlist&maxrecords=250&timespan=1d&sort=datedesc&format=json
