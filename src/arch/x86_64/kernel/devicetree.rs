use dtb::{Reader, StructItem};
use core::fmt;
use core::convert::TryInto;

pub struct MemoryRegion {
    base_address: u64,
    length: u64,
}

impl fmt::Debug for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            write!(
                f,
                "MemoryRegion {{ base_address: {}, length: {} }}",
                self.base_address, self.length
            )
        }
    }
}

impl Default for MemoryRegion {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl MemoryRegion {
    pub fn new(base_addr: u64, length: u64) -> Self {
        Self { 
            base_address: base_addr,
            length: length }
    }

    pub fn base_address(&self) -> u64 {
        self.base_address
    }

    pub fn length(&self) -> u64 {
        self.length
    }
}

pub fn print_information(reader: &Reader<'_>) {
    info!("DEVICE TREE:");

    let mut indent = 0;
    for entry in reader.struct_items() {
        match entry {
            StructItem::BeginNode { name } => {
                info!("{:indent$}{} {{", "", name, indent = indent);
                indent += 2;
            }
            StructItem::EndNode => {
                indent -= 2;
                info!("{:indent$}}}", "", indent = indent);
            }
            StructItem::Property { name, value } => {
                info!("{:indent$}{}: {:?}", "", name, value, indent = indent);
            }
        }
    }
    info! ("End DEVICE TREE");
} 

pub fn get_cpu_count(dtb_addr: usize) -> u32 {
    let reader = unsafe {
        Reader::read_from_address(dtb_addr).unwrap()
    };

    let mut count = 0;
    for entry in reader.struct_items() {
        if entry.is_begin_node() {
            if entry.node_name() == Ok("cpu") {
                count += 1;
            }
        }
    }

    count
}

pub fn get_memory_regions(dtb_addr: usize) -> Option<MemoryRegion> {
    let reader = unsafe {
        Reader::read_from_address(dtb_addr).unwrap()
    };

    let mut reg: &[u8] = &[];

    // a sign indicates that these property are belong to node "memory"
    let mut under_memory = false;

    for entry in reader.struct_items() {
        if entry.node_name() == Ok("memory") {
            // if found a "memory" node, set the sign to true
            under_memory = true;
            continue;
        }

        if under_memory && entry.name() == Ok("reg") {
            reg = entry.value().unwrap();
        }

        if !entry.is_property() {
            // if leave the "memory" node, set the sign to false
            under_memory = false;
        }
    }

    if reg.is_empty() {
        return None;
    }

    // info!("reg: {:?}", reg);
    // let reg_str = core::str::from_utf8(reg).unwrap();
    // info!("reg as str: {}", reg_str);

    // let reg_length = u64::from_be_bytes(reg.take(4..).unwrap().split_at(4).0.try_into().unwrap());
    let reg_length = u32::from_be_bytes(reg[4..8].try_into().unwrap());

    let base_addr = u32::from_be_bytes(reg[0..4].try_into().unwrap());

    info!("mem reg length from dtb: {:x}", reg_length);
    info!("mem base addr from dtb: {:x}", base_addr);

    Some(MemoryRegion {
        base_address: base_addr as u64,
        length: reg_length as u64,
    })
}
