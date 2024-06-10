use crate::utils::{check_password, validate_password};

pub(crate) fn prompt_password<F>(fun: F, compare_password: bool) -> bool where F: Fn(&str) -> bool {
    let mut attempts = 0;
    while attempts < 2  {
        let password = rpassword::prompt_password("Create your password: ").unwrap();
        let password2 = rpassword::prompt_password("Enter your password again: ").unwrap();

        if password.eq(&password2) && validate_password(&password) {
            return if compare_password && check_password(&password) {
                fun(&password)
            } else {
                fun(&password)
            }
        }

        attempts += 1;
    }
    false
}