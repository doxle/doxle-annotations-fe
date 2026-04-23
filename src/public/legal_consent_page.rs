use crate::Route;
use dioxus::prelude::*;

const CSS: &str = include_str!("legal_consent_page.css");

#[component]
pub fn LegalConsentPage() -> Element {
    let mut is_checked = use_signal(|| false);
    let mut show_dialog = use_signal(|| false);
    let nav = use_navigator();
    let nav_for_skip = nav.clone();

    // TODO: restore consent flow — skipping for faster testing
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            nav_for_skip.replace(Route::UploadPlansPage {});
        }
    });

    rsx! {
        style { {CSS} }
        div {
            class: "legal-consent-page",

            // Dialog overlay
            if show_dialog() {
                div {
                    class: "legal-consent-dialog-backdrop",
                    div {
                        class: "legal-consent-dialog-overlay",
                        onclick: move |_| show_dialog.set(false),
                    }
                    div {
                        class: "legal-consent-dialog",
                        span {
                            class: "legal-consent-dialog-message",
                            "You will need to agree to the terms by selecting the checkbox"
                        }
                        button {
                            class: "legal-consent-dialog-btn",
                            onclick: move |_| show_dialog.set(false),
                            "Got it"
                        }
                    }
                }
            }

            // Title
            h1 {
                class: "legal-consent-title",
                "Plans Consent"
            }

            // Legal text card
            div {
                class: "legal-consent-card",
                span {
                    class: "legal-consent-text",
                    "By uploading building plans to Doxle.ai, you agree they will be used solely to improve our \
                     machine learning models. We will not use them for construction or other commercial purposes. \
                     You confirm you have the rights or permissions to upload the plans and acknowledge they \
                     may be protected by copyright. Doxle.ai respects these copyrights and will not claim \
                     ownership or misuse them. You agree to indemnify Doxle Pty Ltd against any claims, damages, or \
                     losses resulting from improper use of the plans or lack of permissions. \
                     For details, see our "
                    Link {
                        class: "legal-consent-link",
                        to: Route::PrivacyPage {},
                        "Terms and Conditions"
                    }
                    "."
                }

                // Checkbox row - entire row is clickable
                div {
                    class: "legal-consent-checkbox-row",
                    onclick: move |_| {
                        is_checked.set(!is_checked());
                    },
                    div {
                        class: if is_checked() { "legal-consent-checkbox checked" } else { "legal-consent-checkbox" },
                    }
                    span {
                        class: "legal-consent-label",
                        "I confirm that I have read and understood the terms above and affirm that I have the \
                         necessary rights and permissions to upload this plan."
                    }
                }
            }

            // Agree / Cancel buttons
            div {
                class: "legal-consent-actions",
                button {
                    class: "legal-consent-cancel-btn",
                    onclick: move |_| {
                        nav.push(Route::HomePage {});
                    },
                    "Cancel"
                }
                button {
                    class: if is_checked() { "legal-consent-agree-btn active" } else { "legal-consent-agree-btn" },
                    onclick: move |_| {
                        if is_checked() {
                            nav.push(Route::UploadPlansPage {});
                        } else {
                            show_dialog.set(true);
                        }
                    },
                    "Agree"
                }
            }
        }
    }
}
