use super::*;

pub(crate) struct LoginModel {
  inner: shared_types::Login,
  failure_message: &'static str,
}
impl LoginModel {
  pub(crate) fn new() -> Self {
    Self {
      inner: shared_types::Login {
        username: String::new(),
        password: String::new(),
        extended: false,
      },
      failure_message: "",
    }
  }
}

pub(crate) enum LoginMsg {
  ToggleExtended,
  UpdateUsername(String),
  UpdatePassword(String),
  Submit,
  LoginSuccess(shared_types::Session),
  LoginError(shared_types::ClientError),
}
pub(crate) fn login_update(msg: LoginMsg, model: &mut LoginModel, orders: &mut impl Orders<Msg>) {
  match msg {
    LoginMsg::ToggleExtended => model.inner.extended = !model.inner.extended,
    LoginMsg::UpdateUsername(x) => model.inner.username = x,
    LoginMsg::UpdatePassword(x) => model.inner.password = x,
    LoginMsg::Submit => {
      let req = Request::new("/api/login")
        .method(Method::Post)
        .json(&model.inner)
      ;
      orders.perform_cmd(async {
        let res: Result<Msg, FetchError> = async {
          let resp = req?.fetch().await?;
          match resp.status().code {
            // If ok, apply the session to state
            200 | 201 => Ok(Msg::Login(LoginMsg::LoginSuccess(resp.json().await?))),
            _   => Ok(Msg::Login(LoginMsg::LoginError(resp.json().await?))),
          }
        }.await;
        match res {
          Ok(x) => Some(x),
          Err(e) => { log!(e); None },
        }
      });
      model.inner.password.clear();
      orders.skip();
    },
    LoginMsg::LoginSuccess(s) => {
      model.inner.username.clear();
      model.inner.extended = false;
      model.failure_message = "";
      orders.send_msg(Msg::Auth(Some(s)));
      orders.skip();
    },
    LoginMsg::LoginError(e) => {
      use shared_types::ClientError;
      model.failure_message = match e {
        ClientError::BadLogin => "Wrong username or password.",
        ClientError::AccountLocked => "Account locked. Contact administrator.",
        _ => "Internal error",
      }
    },
  }
}

pub(crate) fn login_view(model: &LoginModel) -> Node<Msg> {
  div![
    if !model.failure_message.is_empty() {
      div![
        C!["error"],
        br!(),
        &model.failure_message,
        br!(),
      ]
    } else {
      br!()
    },
    "Username: ", br!(),
    input![
      input_ev(Ev::Change, |x| Msg::Login(LoginMsg::UpdateUsername(x))),
      attrs!(At::Value => model.inner.username)
    ],
    br!(),
    "Password: ", br!(),
    input![
      input_ev(Ev::Change, |x| Msg::Login(LoginMsg::UpdatePassword(x))),
      attrs!(At::Value => model.inner.password, At::Type => "password")
    ],
    br!(),
    "Extended session: ",
    input![
      input_ev(Ev::Click, |_| Msg::Login(LoginMsg::ToggleExtended)),
      attrs!(At::Type => "checkbox", At::Checked => model.inner.extended.as_at_value())
    ],
    br!(),
    button![
      "Login",
      ev(Ev::Click, |event| {
        event.prevent_default();
        Msg::Login(LoginMsg::Submit)
      })
    ],
  ]
}
