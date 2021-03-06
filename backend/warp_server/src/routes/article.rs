use warp::{
    filters::BoxedFilter,
    reply::Reply,
    Filter,
};
//use db::Conn;
use db::article::{
    Article,
    ArticleData,
    NewArticle,
};
use identifiers::{
    article::ArticleUuid,
    user::UserUuid,
};
use uuid::Uuid;
use wire::article::{
    ArticlePreviewResponse,
    FullArticleResponse,
    MinimalArticleResponse,
    NewArticleRequest,
    UpdateArticleRequest,
};

use crate::{
    logging::{
        log_attach,
        HttpMethod,
    },
    state::{
        jwt::normal_user_filter,
        State,
    },
    util::{
        convert_and_json,
        convert_vector_and_json,
        json_body_filter,
    },
    uuid_integration::{
        uuid_filter,
        uuid_wrap_filter,
    },
};
use error::Error;
use pool::PooledConn;

pub fn article_api(s: &State) -> BoxedFilter<(impl warp::Reply,)> {
    info!("Attaching Article API");
    warp::path("article")
        .and(
            get_article(s)
                .or(create_article(s))
                .or(update_article(s))
                .or(get_published_articles(s))
                .or(get_owned_unpublished_articles(s))
                .or(publish(s))
                .or(unpublish(s)),
        )
        .with(warp::log("article"))
        .boxed()
}

fn get_article(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Get, "article/<uuid>");

    warp::get2()
        .and(uuid_wrap_filter())
        .and(s.db.clone())
        .and_then(|article_uuid: ArticleUuid, conn: PooledConn| {
            Article::get_article_data(article_uuid, &conn)
                .map(convert_and_json::<ArticleData, FullArticleResponse>)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn get_published_articles(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Get, "article/<index=i32>/<page_size=i32>");
    warp::get2()
        .and(warp::path::param::<i32>())
        .and(warp::path::param::<i32>())
        .and(s.db.clone())
        .and_then(|index: i32, page_size: i32, conn: PooledConn| {
            Article::get_paginated(index, page_size, &conn)
                .map(convert_vector_and_json::<ArticleData, ArticlePreviewResponse>)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn get_owned_unpublished_articles(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Get, "article/owned_unpublished");

    warp::get2()
        .and(warp::path("owned_unpublished"))
        .and(normal_user_filter(s))
        .and(s.db.clone())
        .and_then(|user_uuid: UserUuid, conn: PooledConn| {
            Article::get_unpublished_articles_for_user(user_uuid, &conn)
                .map(convert_vector_and_json::<Article, MinimalArticleResponse>)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn create_article(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Post, "article/");

    warp::post2()
        .and(json_body_filter(128)) // Allow large articles
        .and(normal_user_filter(s))
        .and(s.db.clone())
        .and_then(|request: NewArticleRequest, user_uuid: UserUuid, conn: PooledConn| {
            let mut request: NewArticle = request.into();
            request.author_uuid = user_uuid.0; // This api isn't perfect - so the uuid must be gotten from the jwt

            Article::create_article(request.into(), &conn)
                .map(convert_and_json::<Article, MinimalArticleResponse>)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn update_article(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Put, "article/");

    warp::put2()
        .and(json_body_filter(128))
        .and(normal_user_filter(s))
        .and(s.db.clone())
        .and_then(|request: UpdateArticleRequest, user_uuid: UserUuid, conn: PooledConn| {
            let article_to_update: Article = Article::get_article(request.uuid, &conn).map_err(Error::simple_reject)?;
            if article_to_update.author_uuid != user_uuid.0 {
                return Error::NotAuthorized {
                    reason: "User not author",
                }
                .reject();
            }

            Article::update_article(request.into(), &conn)
                .map(convert_and_json::<Article, MinimalArticleResponse>)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn publish(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Put, "article/publish/<uuid>");

    warp::put2()
        .and(warp::path("publish"))
        .and(uuid_wrap_filter())
        .and(normal_user_filter(s))
        .and(s.db.clone())
        .and_then(|article_uuid: ArticleUuid, user_uuid: UserUuid, conn: PooledConn| {
            let article_to_update: Article = Article::get_article(article_uuid, &conn).map_err(Error::simple_reject)?;
            if article_to_update.author_uuid != user_uuid.0 {
                return Error::NotAuthorized {
                    reason: "User not author",
                }
                .reject();
            }

            Article::set_publish_status(article_uuid, true, &conn)
                .map(|_| warp::http::StatusCode::NO_CONTENT)
                .map_err(Error::simple_reject)
        })
        .boxed()
}

fn unpublish(s: &State) -> BoxedFilter<(impl Reply,)> {
    log_attach(HttpMethod::Put, "article/unpublish/<uuid>");

    warp::put2()
        .and(warp::path("unpublish"))
        .and(uuid_filter())
        .and(normal_user_filter(s))
        .and(s.db.clone())
        .and_then(|uuid: Uuid, user_uuid: UserUuid, conn: PooledConn| {
            let article_uuid = ArticleUuid(uuid);
            let article_to_update: Article = Article::get_article(article_uuid, &conn).map_err(Error::simple_reject)?;
            if article_to_update.author_uuid != user_uuid.0 {
                return Error::NotAuthorized {
                    reason: "User not author",
                }
                .reject();
            }

            Article::set_publish_status(ArticleUuid(uuid), false, &conn)
                .map(|_| warp::http::StatusCode::NO_CONTENT)
                .map_err(Error::simple_reject)
        })
        .boxed()
}
