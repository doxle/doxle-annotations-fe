use dioxus::prelude::*;

#[component]
pub fn PrivacyPage() -> Element {
    rsx! {
        div {
            style: "min-height: 100vh; background: var(--bg-primary); color: var(--text-primary); padding: 60px 24px;",
            div {
                style: "max-width: 720px; margin: 0 auto; font-family: 'HelveticaNeue', Helvetica, Arial, sans-serif;",
                h1 {
                    style: "font-size: 32px; font-weight: 300; margin-bottom: 8px;",
                    "Privacy Policy"
                }
                p {
                    style: "font-size: 14px; color: var(--text-secondary); margin-bottom: 40px;",
                    "Last updated: 7 April 2025"
                }

                // Introduction
                div { style: "margin-bottom: 32px;",
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "Doxle (\"we\", \"our\", \"us\") operates the Doxle mobile application and the website doxle.ai. This Privacy Policy explains how we collect, use, and protect your information when you use our services."
                    }
                }

                // Information We Collect
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Information We Collect" }
                    p { style: "font-size: 15px; line-height: 1.7; margin-bottom: 12px;",
                        "We collect information you provide directly to us, including:"
                    }
                    ul { style: "font-size: 15px; line-height: 1.7; padding-left: 24px;",
                        li { "Account information (name, email address)" }
                        li { "Project and document data you upload" }
                        li { "Usage data and interactions with the service" }
                    }
                }

                // How We Use Your Information
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "How We Use Your Information" }
                    ul { style: "font-size: 15px; line-height: 1.7; padding-left: 24px;",
                        li { "To provide and maintain our service" }
                        li { "To authenticate your account and manage access" }
                        li { "To communicate with you about your account or our services" }
                        li { "To improve and develop new features" }
                    }
                }

                // Data Storage & Security
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Data Storage & Security" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "Your data is stored securely using industry-standard encryption. We use HTTPS for all data transmission and store data on secure cloud infrastructure. We do not sell your personal information to third parties."
                    }
                }

                // Third-Party Services
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Third-Party Services" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "We may use third-party services for authentication, hosting, and analytics. These services have their own privacy policies governing the use of your information."
                    }
                }

                // Data Retention
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Data Retention" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "We retain your data for as long as your account is active or as needed to provide you services. You may request deletion of your account and associated data at any time by contacting us."
                    }
                }

                // Your Rights
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Your Rights" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "You have the right to access, correct, or delete your personal data. You may also request a copy of your data or withdraw consent for data processing at any time."
                    }
                }

                // Children's Privacy
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Children's Privacy" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "Our service is not intended for use by children under the age of 13. We do not knowingly collect personal information from children."
                    }
                }

                // Changes to This Policy
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Changes to This Policy" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "We may update this Privacy Policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the date above."
                    }
                }

                // Contact
                div { style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 20px; font-weight: 500; margin-bottom: 12px;", "Contact Us" }
                    p { style: "font-size: 15px; line-height: 1.7;",
                        "If you have any questions about this Privacy Policy, please contact us at "
                        a {
                            href: "mailto:help@doxle.com",
                            style: "color: #33bfff; text-decoration: none;",
                            "help@doxle.com"
                        }
                    }
                }
            }
        }
    }
}
