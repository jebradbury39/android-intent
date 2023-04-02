use std::borrow::Borrow;
use jni::{errors::Error, objects::{JObject, JString}, JNIEnv, AttachGuard};
use jni::objects::{JValue, JValueOwned};
use jni::sys::jint;
use crate::Flags;

use log::debug;

struct Inner<'env> {
    env: AttachGuard<'env>,
    object: JObject<'env>,
}

/// A messaging object you can use to request an action from another android app component.
#[must_use]
pub struct Intent<'env> {
    inner: Result<Inner<'env>, Error>,
}

impl<'env> Intent<'env> {
    pub fn from_object(env: AttachGuard<'env>, object: JObject<'env>) -> Self {
        Self {
            inner: Ok(Inner { env, object }),
        }
    }

    fn from_fn(f: impl FnOnce() -> Result<Inner<'env>, Error>) -> Self {
        let inner = f();
        Self { inner }
    }

    fn get_static_field_val<'a>(env: &mut AttachGuard<'a>, field_name: impl AsRef<str>, field_type: &str) -> Result<JValueOwned<'a>, Error> {
        debug!("get static field Intent.{} with type {}", field_name.as_ref(), field_type);

        let intent_class = env.find_class("android/content/Intent")?;
        let val = env.get_static_field(&intent_class, field_name.as_ref(), field_type)?;

        return Ok(val);
    }

    pub fn new(mut env: AttachGuard<'env>, action: impl AsRef<str>) -> Self {
        Self::from_fn(|| {
            let action_view = Self::get_static_field_val(&mut env, action.as_ref(), "Ljava/lang/String;")?;

            let intent_class = env.find_class("android/content/Intent")?;
            let intent =
                env.new_object(&intent_class, "(Ljava/lang/String;)V", &[(&action_view).into()])?;

            Ok(Inner {
                env,
                object: intent,
            })
        })
    }

    pub fn new_with_uri(mut env: AttachGuard<'env>, action: impl AsRef<str>, uri: impl AsRef<str>) -> Self {
        Self::from_fn(|| {
            let url_string = env.new_string(uri)?;
            let uri_class = env.find_class("android/net/Uri")?;
            let uri = env.call_static_method(
                uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[(&url_string).into()],
            )?;

            let action_view = Self::get_static_field_val(&mut env, action.as_ref(), "Ljava/lang/String;")?;

            let intent_class = env.find_class("android/content/Intent")?;
            let intent = env.new_object(
                &intent_class,
                "(Ljava/lang/String;Landroid/net/Uri;)V",
                &[(&action_view).into(), (&uri).into()],
            )?;

            Ok(Inner {
                env,
                object: intent,
            })
        })
    }

    /// Add extended data to the intent.
    /// ```no_run
    /// use android_intent::{Action, Extra, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send);
    /// intent.push_extra(Extra::Text, "Hello World!")
    /// # })
    /// ```
    pub fn with_extra(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.and_then(|inner| {
            let mut inner = inner;

            let key = inner.env.new_string(key)?;
            let value = inner.env.new_string(value)?;

            inner.env.call_method(
                &inner.object,
                "putExtra",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                &[(&key).into(), (&value).into()],
            )?;

            Ok(inner)
        })
    }



    /// Builds a new [`Action::Chooser`] Intent that wraps the given target intent.
    /// ```no_run
    /// use android_intent::{Action, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send).into_chhoser();
    /// # })
    /// ```
    pub fn into_chooser(self) -> Self {
        self.into_chooser_with_title(None::<&str>)
    }

    pub fn into_chooser_with_title(self, title: Option<impl AsRef<str>>) -> Self {
        self.and_then(|inner| {
            let title_value: JValueOwned = if let Some(title) = title {
                let s = inner.env.new_string(title)?;
                s.into()
            } else {
                JObject::null().into()
            };
            let mut inner = inner;

            let intent_class = inner.env.find_class("android/content/Intent")?;
            let intent = inner.env.call_static_method(
                &intent_class,
                "createChooser",
                "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
                &[(&inner.object).into(), (&title_value).into()],
            )?;

            inner.object = intent.try_into()?;
            Ok(inner)
        })
    }

    /// Set an explicit MIME data type.
    /// ```no_run
    /// use android_intent::{Action, Intent};
    ///
    /// # android_intent::with_current_env(|env| {
    /// let intent = Intent::new(env, Action::Send);
    /// intent.set_type("text/plain");
    /// # })
    /// ```
    pub fn with_type(self, type_name: impl AsRef<str>) -> Self {
        self.and_then(|inner| {
            let mut inner = inner;
            let jstring = inner.env.new_string(type_name)?;

            inner.env.call_method(
                &inner.object,
                "setType",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[(&jstring).into()],
            )?;

            Ok(inner)
        })
    }

    pub fn add_flags(self, flags: Flags) -> Self {
        self.and_then(|inner| {
            let mut inner = inner;

            let mut jflags: jint = 0;

            for (flag, _) in flags.iter_names() {
                let flag_val = Self::get_static_field_val(&mut inner.env, &format!("FLAG_{}", flag), "I")?;
                let jflag_val: jint = flag_val.i().unwrap();
                jflags |= jflag_val;
            }

            inner.env.call_method(
                &inner.object,
                "addFlags",
                "(I)Landroid/content/Intent;",
                &[jflags.into()],
            )?;

            Ok(inner)
        })
    }

    pub fn add_category(self, category: impl AsRef<str>) -> Self {
        self.and_then(|inner| {
            let mut inner = inner;

            let jcategory = Self::get_static_field_val(&mut inner.env, category.as_ref(), "Ljava/lang/String;")?;

            inner.env.call_method(
                &inner.object,
                "addCategory",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[(&jcategory).into()],
            )?;

            Ok(inner)
        })
    }

    pub fn start_activity(self) -> Result<(), Error> {
        debug!("start_activity");

        let cx = ndk_context::android_context();
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        self.inner.and_then(|inner| {
            let mut inner = inner;

            inner.env.call_method(
                activity,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[(&inner.object).into()],
            )?;

            Ok(())
        })
    }

    pub fn start_activity_for_result(self, request_code: i32) -> Result<(), Error> {
        debug!("start_activity_for_result: {}", request_code);

        let cx = ndk_context::android_context();
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        let jcode: jint = request_code.into();

        self.inner.and_then(|inner| {
            let mut inner = inner;

            inner.env.call_method(
                activity,
                "startActivityForResult",
                "(Landroid/content/Intent;I)V",
                &[(&inner.object).into(), jcode.into()],
            )?;

            Ok(())
        })
    }

    pub fn get_result(self) -> Result<Option<CompletedIntent<'env>>, Error> {
        debug!("get_result for intent");

        let cx = ndk_context::android_context();
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        self.inner.and_then(|inner| {
            let mut inner = inner;

            let jobj = inner.env.call_method(
                activity,
                "getNextIntentResult",
                "(V)Lcom/example/libnumistracker/RustNativeIntentResult;",
                &[],
            )?;

            let jobj = jobj.l().unwrap();

            let jreq_code = inner.env.get_field(&jobj, "requestCode", "I")?;
            let jres_code = inner.env.get_field(&jobj, "resultCode", "I")?;
            let jdata = inner.env.get_field(&jobj, "data", "Landroid/content/Intent;")?;

            let jdata_obj = jdata.l().unwrap();
            if jdata_obj.is_null() {
                debug!("  got null result");
                return Ok(None);
            }

            let intent = Intent::from_object(inner.env, jdata_obj);
            let request_code: i32 = jreq_code.i().unwrap().into();
            let result_code: i32 = jres_code.i().unwrap().into();

            debug!("  got non-null result, request_code={}, result_code={}", request_code, result_code);
            Ok(Some(CompletedIntent {
                request_code,
                result_code,
                data: intent,
            }))
        })
    }

    fn and_then(mut self, f: impl FnOnce(Inner) -> Result<Inner, Error>) -> Self {
        self.inner = match self.inner {
            Ok(inner) => f(inner),
            Err(err) => Err(err),
        };
        self
    }
}

pub struct CompletedIntent<'env> {
    pub request_code: i32,
    pub result_code: i32,
    pub data: Intent<'env>,
}