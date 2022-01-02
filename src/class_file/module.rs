pub struct Module {
    pub name_index: u16,
    pub flags: u16,
    pub version_index: u16,
    pub requirements: Vec<ModuleRequirement>,
    pub exports: Vec<ModuleExport>,
    pub openings: Vec<ModuleOpening>,
    pub uses: Vec<u16>,
    pub provided: Vec<ProvidedModule>
}

pub struct ModuleRequirement {
    pub index: u16,
    pub flags: u16,
    pub version_index: u16
}

pub struct ModuleExport {
    pub index: u16,
    pub flags: u16,
    pub to_indices: Vec<u16>
}

pub struct ModuleOpening {
    pub index: u16,
    pub flags: u16,
    pub to_indices: Vec<u16>
}

pub struct ProvidedModule {
    pub index: u16,
    pub with_indices: Vec<u16>
}
