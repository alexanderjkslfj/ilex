use quick_xml::events::Event;

pub trait GetEvents {
    fn get_all_events(&self) -> Box<dyn Iterator<Item = Event> + '_>;
}
