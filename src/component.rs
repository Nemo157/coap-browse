use vdom_rsjs::VNode;

pub type ShouldRender = bool;

pub trait Component {
    type Action;

    fn render(&self) -> VNode<Self::Action>;

    fn update(&mut self, action: Self::Action) -> ShouldRender;
}
