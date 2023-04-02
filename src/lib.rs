mod action;
pub use action::Action;

mod extra;
pub use extra::Extra;

mod intent;

mod flag;
pub use flag::Flags;

mod category;
pub use category::Category;

pub use intent::Intent;
use jni::{JNIEnv, JavaVM, AttachGuard};
use ndk_context::AndroidContext;

pub struct IntentEnv {
    cx: AndroidContext,
    vm: JavaVM,
}

impl IntentEnv {
    pub fn new() -> Self {
        let cx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }.unwrap();

        return Self {
            cx,
            vm,
        };
    }

    pub fn get_env(&mut self) -> AttachGuard {
        return self.vm.attach_current_thread().unwrap();
    }
}

/// Run 'f' with the current [`JNIEnv`] from [`ndk_context`].
pub fn with_current_env(intent_env: &mut IntentEnv, f: impl FnOnce(AttachGuard)) {
    let env = intent_env.get_env();

    f(env);
}
