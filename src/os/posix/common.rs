use crate::Modifiers;
use crate::Result;
use crate::{Key, MenuHandle, MenuItem, MenuItemHandle, PosixMenu, PosixMenuItem};

pub struct Menu {
    pub internal: PosixMenu,
}

impl Menu {
    pub fn new(name: &str) -> Result<Menu> {
        Ok(Menu {
            internal: PosixMenu {
                handle: MenuHandle(0),
                item_counter: MenuItemHandle(0),
                name: name.to_owned(),
                items: Vec::new(),
            },
        })
    }

    pub fn add_sub_menu(&mut self, name: &str, sub_menu: &Menu) {
        let handle = self.next_item_handle();
        self.internal.items.push(PosixMenuItem {
            label: name.to_owned(),
            handle,
            sub_menu: Some(Box::new(sub_menu.internal.clone())),
            id: 0,
            enabled: true,
            key: Key::Unknown,
            modifiers: Modifiers::empty(),
        });
    }

    fn next_item_handle(&mut self) -> MenuItemHandle {
        let handle = self.internal.item_counter;
        self.internal.item_counter.0 += 1;
        handle
    }

    pub fn add_menu_item(&mut self, item: &MenuItem) -> MenuItemHandle {
        let item_handle = self.next_item_handle();
        self.internal.items.push(PosixMenuItem {
            sub_menu: None,
            handle: self.internal.item_counter,
            id: item.id,
            label: item.label.clone(),
            enabled: item.enabled,
            key: item.key,
            modifiers: item.modifiers,
        });
        item_handle
    }

    pub fn remove_item(&mut self, handle: &MenuItemHandle) {
        self.internal.items.retain(|item| item.handle.0 != handle.0);
    }
}
