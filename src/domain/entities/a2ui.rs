use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2UIComponent {
    pub id: String,
    pub component: String, // Type e.g., "Text", "Button"
    #[serde(flatten)]
    pub properties: serde_json::Value,
    pub bindings: Option<HashMap<String, String>>,
    pub actions: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Surface {
    pub id: String,
    pub components: Vec<A2UIComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum A2UIMessage {
    #[serde(rename_all = "camelCase")]
    SurfaceUpdate {
        surface_id: String,
        components: Vec<A2UIComponent>,
    },
    #[serde(rename_all = "camelCase")]
    DataModelUpdate {
        surface_id: String,
        data: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    UserAction {
        surface_id: String,
        action_id: String,
        component_id: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataModel {
    pub values: HashMap<String, serde_json::Value>,
}

impl Surface {
    pub fn new(id: String) -> Self {
        Self {
            id,
            components: Vec::new(),
        }
    }

    pub fn update_components(&mut self, new_components: Vec<A2UIComponent>) {
        for new_comp in new_components {
            if let Some(pos) = self.components.iter().position(|c| c.id == new_comp.id) {
                self.components[pos] = new_comp;
            } else {
                self.components.push(new_comp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_surface_update() {
        let mut surface = Surface::new("test_id".to_string());
        let comp1 = A2UIComponent {
            id: "comp1".to_string(),
            component: "Text".to_string(),
            properties: json!({"text": "Hello"}),
            bindings: None,
            actions: None,
        };

        surface.update_components(vec![comp1.clone()]);
        assert_eq!(surface.components.len(), 1);
        assert_eq!(surface.components[0].id, "comp1");

        let comp1_upd = A2UIComponent {
            id: "comp1".to_string(),
            component: "Text".to_string(),
            properties: json!({"text": "Updated"}),
            bindings: None,
            actions: None,
        };

        surface.update_components(vec![comp1_upd]);
        assert_eq!(surface.components.len(), 1);
        assert_eq!(surface.components[0].properties, json!({"text": "Updated"}));
    }

    #[test]
    fn test_message_deserialization() {
        let json = json!({
            "type": "surfaceUpdate",
            "payload": {
                "surfaceId": "s1",
                "components": [
                    {
                        "id": "c1",
                        "component": "Button",
                        "label": "Click me"
                    }
                ]
            }
        });

        let msg: A2UIMessage = serde_json::from_value(json).unwrap();
        if let A2UIMessage::SurfaceUpdate {
            surface_id,
            components,
        } = msg
        {
            assert_eq!(surface_id, "s1");
            assert_eq!(components.len(), 1);
            assert_eq!(components[0].component, "Button");
        } else {
            panic!("Wrong message type");
        }
    }
}
