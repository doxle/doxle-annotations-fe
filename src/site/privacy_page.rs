use dioxus::prelude::*;

const PRIVACY_CSS: &str = include_str!("privacy.css");
const DOG_BLUE_ICON: Asset = asset!("/assets/icons/dog-blue.svg");

#[component]
pub fn PrivacyPage() -> Element {
    let nav = navigator();
    rsx! {
        style { {PRIVACY_CSS} }
        div {
            class: "privacy-page",
            div {
                class: "privacy-content",
                div {
                    class: "privacy-header",
                    onclick: move |_| { nav.push(crate::Route::HomePage {}); },
                    img { class: "privacy-logo", src: DOG_BLUE_ICON, alt: "Doxle" }
                }
                h1 { class: "privacy-title", "Privacy Policy" }
                p { class: "privacy-updated", "Last updated: 7 April 2025" }

                div { class: "privacy-section",
                    p { "Doxle (\"we\", \"our\", \"us\") operates the Doxle mobile application and the website doxle.ai. This Privacy Policy explains how we collect, use, and protect your information when you use our services." }
                }

                div { class: "privacy-section",
                    h2 { "Information We Collect" }
                    p { "We collect information you provide directly to us, including:" }
                    ul {
                        li { "Account information (name, email address)" }
                        li { "Project and document data you upload" }
                        li { "Usage data and interactions with the service" }
                    }
                }

                div { class: "privacy-section",
                    h2 { "How We Use Your Information" }
                    ul {
                        li { "To provide and maintain our service" }
                        li { "To authenticate your account and manage access" }
                        li { "To communicate with you about your account or our services" }
                        li { "To improve and develop new features" }
                    }
                }

                div { class: "privacy-section",
                    h2 { "Data Storage & Security" }
                    p { "Your data is stored securely using industry-standard encryption. We use HTTPS for all data transmission and store data on secure cloud infrastructure. We do not sell your personal information to third parties." }
                }

                div { class: "privacy-section",
                    h2 { "Third-Party Services" }
                    p { "We may use third-party services for authentication, hosting, and analytics. These services have their own privacy policies governing the use of your information." }
                }

                div { class: "privacy-section",
                    h2 { "Data Retention" }
                    p { "We retain your data for as long as your account is active or as needed to provide you services. You may request deletion of your account and associated data at any time by contacting us." }
                }

                div { class: "privacy-section",
                    h2 { "Your Rights" }
                    p { "You have the right to access, correct, or delete your personal data. You may also request a copy of your data or withdraw consent for data processing at any time." }
                }

                div { class: "privacy-section",
                    h2 { "Children's Privacy" }
                    p { "Our service is not intended for use by children under the age of 13. We do not knowingly collect personal information from children." }
                }

                div { class: "privacy-section",
                    h2 { "Changes to This Policy" }
                    p { "We may update this Privacy Policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the date above." }
                }

                div { class: "privacy-section",
                    h2 { "Contact Us" }
                    p {
                        "If you have any questions about this Privacy Policy, please contact us at "
                        a { href: "mailto:help@doxle.com", "help@doxle.com" }
                    }
                }
            }
        }
    }
}
