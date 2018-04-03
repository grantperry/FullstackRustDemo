use yew::prelude::*;
use Context;
use yew::format::{Json};

use yew::services::fetch::{FetchTask, Response};

use components::markdown::author_markdown_toggle::AuthorMarkdownToggle;
use components::button::Button;

use requests_and_responses::thread::{NewThreadRequest, ThreadResponse};
use requests_and_responses::post::NewPostRequest;
use datatypes::forum::ForumData;
use failure::Error;

use context::networking::*;

pub struct NewThread {
    title: String,
    post_content: String,
    forum: ForumData,
    callback: Option<Callback<()>>,
    ft: Option<FetchTask>
}


pub enum Msg {
    CreateNewThread,
    UpdatePostContent(String),
    UpdateThreadTitle(String),
    NoOp
}

#[derive(Clone, PartialEq)]
pub struct Props {
    pub callback: Option<Callback<()>>,
    pub forum: ForumData
}

impl Default for Props {
    fn default() -> Self {
        Props {
            callback: None,
            forum: ForumData::default() // I don't like this, possibly make it optional and set it to none by default
        }
    }
}

impl Component<Context> for NewThread {
    type Msg = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _context: &mut Env<Context, Self>) -> Self {

        NewThread {
            title: String::default(),
            forum: props.forum,
            post_content: String::default(),
            callback: props.callback,
            ft: None
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::CreateNewThread => {
                let callback = context.send_back(|response: Response<Json<Result<ThreadResponse, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}, {:?}", meta, data);
                    Msg::NoOp
                });

                if let Ok(user_id) = context.user_id() {
                    let new_thread_request = NewThreadRequest {
                        forum_id: self.forum.id,
                        author_id: user_id,
                        title: self.title.clone(),
                        post_content: self.post_content.clone()
                    };

                    let task = context.make_request(RequestWrapper::CreateThread(new_thread_request), callback);
                    // TODO: Redirect to login on error
                    self.ft = task.ok();


                    if let Some(ref cb) = self.callback {
                        cb.emit(());
                    }
                } else {
                    eprintln!("Couldn't get user id required for request")
                }
                false
            }
            Msg::UpdateThreadTitle(title) => {
                self.title = title;
                true
            }
            Msg::UpdatePostContent(text) => {
                self.post_content = text;
                true
            }
            Msg:: NoOp => {
                false
            }

        }
    }

    fn change(&mut self, _props: Self::Properties, _: &mut Env<Context, Self>) -> ShouldRender {
        true
    }
}

impl Renderable<Context, NewThread> for NewThread {

    fn view(&self) -> Html<Context, Self> {

        return html! {
            <div>
                <input
                    class="form-control",
                //    disabled=self.disabled,
                    placeholder="Thread Title",
                    value=&self.title,
                    oninput=|e: InputData| Msg::UpdateThreadTitle(e.value),
//                    onkeypress=|e: KeyData| {
//                        if e.key == "Enter" { Msg::Submit } else {Msg::NoOp}
//                    },
                 />
                 <AuthorMarkdownToggle: callback=|text| Msg::UpdatePostContent(text), />
                 <Button: onclick=|_| Msg::CreateNewThread, />

            </div>
        }
    }
}