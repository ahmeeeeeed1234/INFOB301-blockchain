pub trait Parenting {
    fn is_parent(&self, parent_id: &[u8]) -> bool;
}

#[derive(Debug, Default)]
pub struct TreeNode<T: Default + Parenting> {
    value: T,
    children: Vec<TreeNode<T>>,
}

impl<T: Default + Parenting> TreeNode<T> {

    pub fn new(value: T) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) {
        self.children.push(TreeNode::new(value));
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, value: &T)
    where
        T: PartialEq,
    {
        self.children.retain_mut(|child| {
            child.remove(value);
            &child.value != value
        });
    }

    pub fn depth(&self) -> usize {
        let mut max_depth = 0;

        for child in &self.children {
            let child_depth = child.depth();

            if child_depth > max_depth {
                max_depth = child_depth;
            }
        }

        max_depth + 1
    }

    pub fn look_for_parent(&mut self, parent_id: &[u8]) -> Option<&mut TreeNode<T>> {
        if self.value.is_parent(parent_id) {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.look_for_parent(parent_id) {
                return Some(found);
            }
        }

        None
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn children(&self) -> &Vec<TreeNode<T>> {
        &self.children
    }

    #[allow(dead_code)]
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Default, Debug)]
    struct Data {
        val: u32,
        parent_id: [u8; 1],
    }

    impl Data {
        fn new(val: u32, parent_id: [u8; 1]) -> Self {
            Data { val, parent_id }
        }
    }

    impl Parenting for Data {
        fn is_parent(&self, parent_id: &[u8]) -> bool {
            self.val as u8 == parent_id[0]
        }
    }

    #[test]
    fn test_tree_operations() {
        let mut root = TreeNode::new(Data::new(42, [0]));

        root.insert(Data::new(2, [42]));
        root.insert(Data::new(3, [42]));

        root.children_mut()[0].insert(Data::new(4, [2]));
        root.children_mut()[0].insert(Data::new(5, [2]));
        root.children_mut()[1].insert(Data::new(6, [3]));

        assert_eq!(root.depth(), 3);

        assert_eq!(
            root.look_for_parent(&[3]).unwrap().value(),
            &Data::new(3, [42])
        );

        root.remove(&Data::new(2, [42]));

        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].value(), &Data::new(3, [42]));
    }
}