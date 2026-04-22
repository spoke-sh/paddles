use paddles::domain::model::TurnEvent;
use paddles::infrastructure::runtime_presentation::project_runtime_event;

#[test]
fn runtime_event_presentation_is_available_outside_the_domain_boundary() {
    let presentation = project_runtime_event(&TurnEvent::RouteSelected {
        summary: "direct answer".to_string(),
    });

    assert_eq!(presentation.badge, "route");
    assert_eq!(presentation.badge_class, "route");
    assert_eq!(presentation.title, "• Routed");
    assert_eq!(presentation.detail, "direct answer");
    assert_eq!(presentation.text, "direct answer");
}
