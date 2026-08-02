macro_rules! events {
    {
        $(
            $event_name: ident => $event: path
        ),*
    } => {
        #[derive(Debug, Clone)]
        pub enum Event {
            $(
                $event_name($event)
            ),*
        }

        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum EventKind {
            $(
                $event_name
            ),*
        }

        $(
            impl $crate::AbstractEvent for $event {
                const KIND: EventKind = $crate::EventKind::$event_name;
            }

            impl TryFrom<$crate::Event> for $event {
                type Error = $crate::EventRoutingError;

                fn try_from(e: $crate::Event) -> Result<Self, Self::Error> {
                    match e {
                        $crate::Event::$event_name(e) => Ok(e),
                        _ => Err($crate::EventRoutingError)
                    }
                }
            }

            impl From<$event> for $crate::Event {
                fn from(e: $event) -> Self {
                    $crate::Event::$event_name(e)
                }
            }
        )*
    };
}
