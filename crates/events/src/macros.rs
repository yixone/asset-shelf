#[macro_export]
macro_rules! event {
    {
        $( #[$meta: meta] )*
        $event_name: ident {
            $(
                $( #[$data_meta: meta] )*
                $event_data_id: ident: $event_data_ty: ty
            ),* $(,)?
        }
    } => {
        $( #[$meta] )*
        #[derive(Debug)]
        pub struct $event_name {
            $(
                $( #[$data_meta] )*
                pub $event_data_id: $event_data_ty
            ),*
        }

        impl $crate::DynamicEvent for $event_name {}
    };
}
