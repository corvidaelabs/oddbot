use serenity::all::CreateCommand;

pub fn register_character() -> CreateCommand {
    CreateCommand::new("register").description("Register an Oblivion character")
}

pub fn get_character() -> CreateCommand {
    CreateCommand::new("whoami").description("What Oblivion character am I?")
}

pub fn delete_character() -> CreateCommand {
    CreateCommand::new("die").description("Delete your Oblivion character")
}
