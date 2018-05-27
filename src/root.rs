use std::collections::HashMap;

use vdom_rsjs::{VNode, VTag, VProperty};
use tokio_core::reactor::Handle;

use component::{ShouldRender, Component};


#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Increment,
    Decrement,
}

#[derive(Debug, Clone)]
pub struct Root {
    count: usize,
}

impl Root {
    pub fn new(_handle: Handle) -> Root {
        Root { count: 0 }
    }
}

impl Component for Root {
    type Action = Action;

    fn update(&mut self, action: Action) -> ShouldRender {
        match action {
            Action::Increment => self.count += 1,
            Action::Decrement => self.count -= 1,
        }
        true
    }

    fn render(&self) -> VNode<Action> {
        VNode::Tag(VTag {
            name: "div".into(),
            properties: HashMap::new(),
            children: vec![
                VNode::Text(self.count.to_string()),
                VNode::Tag(VTag {
                    name: "br".into(),
                    properties: HashMap::new(),
                    children: vec![],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "button".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("onclick".into(), VProperty::Action(Action::Increment));
                        props
                    },
                    children: vec![
                        VNode::Text("increment".into()),
                    ],
                    key: None,
                    namespace: None,
                }),
                VNode::Tag(VTag {
                    name: "button".into(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("onclick".into(), VProperty::Action(Action::Decrement));
                        props
                    },
                    children: vec![
                        VNode::Text("decrement".into()),
                    ],
                    key: None,
                    namespace: None,
                }),
            ],
            key: None,
            namespace: None,
        })
    }
}

