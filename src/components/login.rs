use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    
    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        // TODO: Implement authentication with backend
        println!("Login attempt - Email: {}, Password: [REDACTED]", email());
    };
    
    rsx! {
        div { class: "min-h-screen bg-gray-900 flex items-center justify-center px-4",
            div { class: "bg-gray-800 p-8 rounded-lg shadow-xl w-full max-w-md",
                // Back button
                Link {
                    to: Route::HomePage {},
                    class: "text-gray-400 hover:text-white mb-6 flex items-center",
                    "← Back"
                }
                
                h2 { class: "text-3xl font-bold text-white mb-2 text-center",
                    "Welcome Back"
                }
                p { class: "text-gray-400 mb-8 text-center",
                    "Sign in to your account"
                }
                
                form {
                    onsubmit: handle_submit,
                    
                    // Email field
                    div { class: "mb-4",
                        label { 
                            class: "block text-sm font-medium text-gray-400 mb-2",
                            "Email Address"
                        }
                        input {
                            class: "w-full px-4 py-3 bg-gray-700 text-white rounded-lg border border-gray-600 focus:outline-none focus:border-blue-500 transition-colors",
                            r#type: "email",
                            placeholder: "you@example.com",
                            required: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }
                    
                    // Password field
                    div { class: "mb-6",
                        label { 
                            class: "block text-sm font-medium text-gray-400 mb-2",
                            "Password"
                        }
                        input {
                            class: "w-full px-4 py-3 bg-gray-700 text-white rounded-lg border border-gray-600 focus:outline-none focus:border-blue-500 transition-colors",
                            r#type: "password",
                            placeholder: "••••••••",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }
                    
                    // Submit button
                    button {
                        class: "w-full px-4 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-semibold",
                        r#type: "submit",
                        "Sign In"
                    }
                }
                
                // Sign up link
                p { class: "text-center text-gray-400 mt-6 text-sm",
                    "Don't have an account? "
                    span { class: "text-blue-500 hover:text-blue-400 cursor-pointer",
                        "Sign up"
                    }
                }
            }
        }
    }
}
