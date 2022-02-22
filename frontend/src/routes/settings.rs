use super::*;

pub(crate) struct SettingsModel {
  inner: shared_types::PasswordChange,
  new_password_verification: String,
  failure_message: &'static str,
  success_message: &'static str,
}
impl SettingsModel {
  pub(crate) fn new() -> Self {
    Self {
      inner: shared_types::PasswordChange {
        old_password: String::new(),
        new_password: String::new(),
        clear_sessions: false,
      },
      new_password_verification: String::new(),
      failure_message: "",
      success_message: "",
    }
  }
}

pub(crate) enum SettingsMsg {
  SetOldPassword(String),
  SetNewPassword(String),
  SetNewPasswordVerification(String),
  ToggleClearSessions,
  PasswordChangeSubmit,
  PasswordChangeSuccess(bool),
  PasswordChangeError(shared_types::ClientError),
}
pub(crate) fn settings_update(
  msg: SettingsMsg,
  model: &mut SettingsModel,
  session: &shared_types::Session,
  orders: &mut impl Orders<Msg>,
) {
  match msg {
    SettingsMsg::ToggleClearSessions => model.inner.clear_sessions = !model.inner.clear_sessions,
    SettingsMsg::SetOldPassword(x) => model.inner.old_password = x,
    SettingsMsg::SetNewPassword(x) => model.inner.new_password = x,
    SettingsMsg::SetNewPasswordVerification(x) => model.new_password_verification = x,
    SettingsMsg::PasswordChangeSubmit => {
      if model.new_password_verification == model.inner.new_password {
        let req = Request::new("/api/user/password")
          .method(Method::Post)
          .header(Header::bearer(session.key.clone()))
          .json(&model.inner);
        let clear_sessions = model.inner.clear_sessions;
        orders.perform_cmd(async move {
          let res: Result<SettingsMsg, FetchError> = async {
            let resp = req?.fetch().await?;
            match resp.status().code {
              204 => Ok(SettingsMsg::PasswordChangeSuccess(clear_sessions)),
              _ => Ok(SettingsMsg::PasswordChangeError(resp.json().await?)),
            }
          }
          .await;
          match res {
            Ok(x) => Some(Msg::Routes(RoutesMsg::Settings(x))),
            Err(e) => {
              log!("Error occured in password change request", e);
              None
            }
          }
        });
        let tmp = model.inner.clear_sessions;
        *model = SettingsModel::new();
        model.inner.clear_sessions = tmp;
        orders.skip(); // Let the result of the interaction cause re-render instead
      } else {
        model.new_password_verification.clear();
        model.inner.new_password.clear();
        model.failure_message = "New password and new password confirmation didn't match.";
      }
    }
    SettingsMsg::PasswordChangeSuccess(clear_sessions) => {
      if !clear_sessions {
        model.success_message = "Password changed";
      } else {
        orders.send_msg(Msg::ClearAuth("Password changed and all sessions cleared"));
        orders.skip();
      }
    }
    SettingsMsg::PasswordChangeError(err) => {
      use shared_types::ClientError;
      model.failure_message = match err {
        ClientError::Unauthorized => "Old password was wrong",
        ClientError::BadPassword => "New password doesn't fulfil password requirements",
        _ => {
          log!("Password change error:", err);
          "Internal error"
        }
      }
    }
  }
}

pub(crate) fn settings_view(model: &SettingsModel) -> Node<SettingsMsg> {
  div![
    C!["password_change"],
    if !model.success_message.is_empty() {
      div![C!["notice"], br!(), &model.success_message, br!(),]
    } else {
      Node::Empty
    },
    if !model.failure_message.is_empty() {
      div![C!["error"], br!(), &model.failure_message, br!(),]
    } else {
      Node::Empty
    },
    form![
      "Old password:",
      br!(),
      input![
        input_ev(Ev::Change, SettingsMsg::SetOldPassword),
        attrs!(At::Value => model.inner.old_password, At::Type => "password")
      ],
      br!(),
      "New password:",
      br!(),
      input![
        input_ev(Ev::Change, SettingsMsg::SetNewPassword),
        attrs!(At::Value => model.inner.new_password, At::Type => "password")
      ],
      br!(),
      "Confirm new password:",
      br!(),
      input![
        input_ev(Ev::Change, SettingsMsg::SetNewPasswordVerification),
        attrs!(At::Value => model.new_password_verification, At::Type => "password")
      ],
      br!(),
      "Clear existing sessions:",
      br!(),
      input![
        input_ev(Ev::Click, |_| SettingsMsg::ToggleClearSessions),
        attrs!(At::Checked => model.inner.clear_sessions.as_at_value(), At::Type => "checkbox")
      ],
      br!(),
      input![attrs!(At::Value => "Submit", At::Type => "submit"),],
      ev(Ev::Submit, |event| {
        event.prevent_default();
        SettingsMsg::PasswordChangeSubmit
      })
    ]
  ]
}
