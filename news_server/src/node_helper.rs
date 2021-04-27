// use html5ever::serialize::{SerializeOpts, TraversalScope};
use html5ever::{LocalName, Namespace, QualName};
use kuchiki::iter::NodeIterator;
use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
use std::{collections::HashMap, str, sync::Mutex};

pub struct DomHelper {
    selector_cache: Mutex<HashMap<String, kuchiki::Selectors>>,
}

impl DomHelper {
    pub fn new() -> Self {
        DomHelper {
            selector_cache: Mutex::new(HashMap::default()),
        }
    }

    pub fn parse_html(&self, html: &str) -> NodeRef {
        kuchiki::parse_html().one(html)
    }

    #[inline(always)]
    pub fn select<S>(&self, node: &NodeRef, selector: S) -> Vec<NodeRef>
    where
        S: Into<String>,
    {
        let selector_raw = selector.into();
        let mut cache = self.selector_cache.lock().unwrap();
        let selectors = cache.entry(selector_raw.clone()).or_insert(
            kuchiki::Selectors::compile(&selector_raw)
                .expect(&format!("Wrong selector: {}", &selector_raw)),
        );

        selectors
            .filter(node.inclusive_descendants().elements())
            .map(|e| e.as_node().clone())
            .collect::<Vec<_>>()
    }

    pub fn create_new_element(&self, tag_name: &str) -> NodeRef {
        let attributes: HashMap<kuchiki::ExpandedName, kuchiki::Attribute> = HashMap::default();
        let name = QualName::new(None, Namespace::from(""), LocalName::from(tag_name));
        NodeRef::new_element(name, attributes)
    }

    pub fn wrap_with_element(&self, node_to_wrap: &NodeRef, wrap_tag_name: &str) -> NodeRef {
        let new_element = self.create_new_element(wrap_tag_name);
        node_to_wrap.insert_after(new_element.clone());
        node_to_wrap.detach();
        new_element.append(node_to_wrap.clone());
        new_element
    }

    pub fn set_node_attribute(
        &self,
        node: &kuchiki::NodeRef,
        attribute: &str,
        value: &str,
    ) -> bool {
        if let Some(element) = node.as_element() {
            let attr = kuchiki::Attribute {
                prefix: None,
                value: value.to_string(),
            };
            *element
                .attributes
                .borrow_mut()
                .entry(attribute)
                .or_insert(attr.clone()) = attr.clone();
            return true;
        }

        false
    }

    pub fn rename_node(&self, node: &NodeRef, new_tag_name: &str) {
        let new_node = self.create_new_element(new_tag_name);
        if let Some(element) = node.as_element() {
            let old_attributes = element.attributes.borrow().clone();
            *new_node.as_element().unwrap().attributes.borrow_mut() = old_attributes;
        }

        for child in node.children() {
            child.detach();
            new_node.append(child);
        }
        node.insert_after(new_node);
        node.detach();
    }
}
