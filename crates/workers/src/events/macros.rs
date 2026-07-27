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
            impl $crate::events::AbstractEvent for $event {
                const KIND: EventKind = $crate::events::EventKind::$event_name;
            }

            impl TryFrom<$crate::events::Event> for $event {
                type Error = $crate::events::EventRoutingError;

                fn try_from(e: $crate::events::Event) -> Result<Self, Self::Error> {
                    match e {
                        $crate::events::Event::$event_name(e) => Ok(e),
                        _ => Err($crate::events::EventRoutingError)
                    }
                }
            }

            impl From<$event> for $crate::events::Event {
                fn from(e: $event) -> Self {
                    $crate::events::Event::$event_name(e)
                }
            }
        )*
    };
}

macro_rules! app_event {
    {
        $( #[$meta: meta] )*
        $event_name: ident
    } => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $event_name;
    };
    {
        $( #[$meta: meta] )*
        $event_name: ident {
            $(
                $( #[$fmeta: meta] )*
                $f: ident: $ft: ty
            ),*
        }
    } => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $event_name {
            $(
                $( #[$fmeta] )*
                pub $f: $ft
            ),*
        }
    };
}
