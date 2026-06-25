use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::{
    dioxus_attributes::attributes,
    merge_attributes,
    navbar::{
        self, NavbarContentProps, NavbarItemProps, NavbarNavProps, NavbarProps, NavbarTriggerProps,
    },
};
#[css_module("/src/components/navbar/style.css")]
struct Styles;

#[component]
pub fn Navbar(props: NavbarProps) -> Element {
    let base = attributes!(div {
        class: Styles::dx_navbar,
    });

    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        navbar::Navbar {
            disabled: props.disabled,
            roving_loop: props.roving_loop,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn NavbarNav(props: NavbarNavProps) -> Element {
    rsx! {
        navbar::NavbarNav {
            class: Styles::dx_navbar_nav,
            index: props.index,
            disabled: props.disabled,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn NavbarTrigger(props: NavbarTriggerProps) -> Element {
    rsx! {
        navbar::NavbarTrigger { class: Styles::dx_navbar_trigger, attributes: props.attributes,
            {props.children}
            ChevronDown {
                class: Styles::dx_navbar_expand_icon,
                size: "20px",
                stroke: "var(--color-secondary-4)",
            }
        }
    }
}

#[component]
pub fn NavbarContent(props: NavbarContentProps) -> Element {
    rsx! {
        navbar::NavbarContent {
            class: Styles::dx_navbar_content,
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    rsx! {
        navbar::NavbarItem {
            class: Styles::dx_navbar_item.to_string(),
            index: props.index,
            value: props.value,
            disabled: props.disabled,
            new_tab: props.new_tab,
            to: props.to,
            active_class: props.active_class,
            attributes: props.attributes,
            on_select: props.on_select,
            onclick: props.onclick,
            onmounted: props.onmounted,
            {props.children}
        }
    }
}
