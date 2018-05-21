
use yew::services::route::*;

#[derive(Debug, PartialEq, Clone)]
pub enum BucketRoute {
    BucketList,
    Bucket{bucket_id: i32},
    Create
}

impl Default for BucketRoute {
    fn default() -> Self {
        BucketRoute::BucketList
    }
}

impl Router for BucketRoute {
    fn to_route(&self) -> RouteInfo {
        use self::BucketRoute::*;
        match *self {
            BucketList => RouteInfo::parse("/").unwrap(),
            Bucket{bucket_id} => RouteInfo::parse(&format!("/{}", bucket_id)).unwrap(),
            Create => RouteInfo::parse("/create").unwrap()
        }
    }
    fn from_route(route: &mut RouteInfo) -> Option<Self> {
        use self::BucketRoute::*;
        if let Some(RouteSection::Node { segment }) = route.next() {
            if let Ok(bucket_id) = segment.parse::<i32>() {
                Some(Bucket{bucket_id})
            } else if segment == "create" {
                Some(Create)
            } else {
                Some(BucketList)
            }
        } else {
            None
        }
    }
}