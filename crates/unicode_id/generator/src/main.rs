use std::{
  collections::{HashMap, HashSet},
  error::Error,
  fs::File,
  io::{self, BufRead, Write},
};

const ROW_SIZE: usize = 64;

fn main() -> Result<(), Box<dyn Error>> {
  let (charset_id_start, charset_id_continue) = generate_charsets();
  let mut unmerged_chunks: Vec<(u8, Vec<u8>)> = Vec::new();

  let (max_bound_start, mut id_start_table) =
    gen_data_table(&mut unmerged_chunks, &charset_id_start);

  let (max_bound_continue, mut id_continue_table) =
    gen_data_table(&mut unmerged_chunks, &charset_id_continue);

  let (full_data_chunk, chunk_index_map) = merge_chunks(unmerged_chunks);

  for i in 0..id_start_table.len() {
    id_start_table[i] = *chunk_index_map
      .get(&id_start_table[i])
      .expect("Missing chunk index");
  }

  for i in 0..id_continue_table.len() {
    id_continue_table[i] = *chunk_index_map
      .get(&id_continue_table[i])
      .expect("Missing chunk index");
  }

  for c in 0..=max_bound_start {
    let chunk_i = &id_start_table[c as usize / ROW_SIZE / 8];
    let byte = &full_data_chunk[*chunk_i as usize * ROW_SIZE / 2 + c as usize / 8 % ROW_SIZE];
    let id_start = (byte.wrapping_shr(c % 8) & 1) == 1;

    if id_start != charset_id_start.contains(&c) {
      // if id_start != unicode_id_start::is_id_start(char::from_u32(c).unwrap()) {
      Err("Invalid lol.")?;
    }
  }

  for c in 0..=max_bound_continue {
    let chunk_i = &id_continue_table[c as usize / ROW_SIZE / 8];
    let byte = &full_data_chunk[*chunk_i as usize * ROW_SIZE / 2 + c as usize / 8 % ROW_SIZE];
    let id_continue = (byte.wrapping_shr(c % 8) & 1) == 1;

    if id_continue != charset_id_continue.contains(&c) {
      // if id_continue != unicode_id_start::is_id_continue(char::from_u32(c).unwrap()) {
      Err("Invalid lol.")?;
    }
  }

  let mut out_file = File::create("../src/lib.rs")?;

  out_file.write(b"pub const ROW_SIZE: usize = ")?;
  out_file.write(ROW_SIZE.to_string().as_bytes())?;
  out_file.write(b";\n\npub const MAX_BOUND_ID_START: u32 = ")?;
  out_file.write(format!("{max_bound_start:#08X}").as_bytes())?;
  out_file.write(b";\n\npub const MAX_BOUND_ID_CONTINUE: u32 = ")?;
  out_file.write(format!("{max_bound_continue:#08X}").as_bytes())?;
  out_file.write(b";\n\npub const ID_START_TABLE: [u8;")?;
  out_file.write(&id_start_table.len().to_string().as_bytes())?;
  out_file.write(b"] = [")?;

  for (i, index) in id_start_table.iter().enumerate() {
    if i > 0 {
      out_file.write(b", ")?;
    }
    out_file.write(format!("{index:#04X}").as_bytes())?;
  }

  out_file.write(b"];\n\npub const ID_CONTINUE_TABLE: [u8;")?;
  out_file.write(&id_continue_table.len().to_string().as_bytes())?;
  out_file.write(b"] = [")?;

  for (i, index) in id_continue_table.iter().enumerate() {
    if i > 0 {
      out_file.write(b", ")?;
    }
    out_file.write(format!("{index:#04X}").as_bytes())?;
  }

  out_file.write(b"];\n\npub const DATA_TABLE: [u8;")?;
  out_file.write(&full_data_chunk.len().to_string().as_bytes())?;
  out_file.write(b"] = [")?;

  for (i, byte) in full_data_chunk.iter().enumerate() {
    if i > 0 {
      out_file.write(b", ")?;
    }
    out_file.write(format!("{byte:#04X}").as_bytes())?;
  }

  out_file.write(b"];\n")?;

  println!("Wrote to ../src/lib.rs");

  println!(
    "ID_Start table indexes: {} ({} KB)",
    id_start_table.len(),
    id_start_table.len() as f64 / 1024.0
  );

  println!(
    "ID_Continue table indexes: {} ({} KB)",
    id_continue_table.len(),
    id_continue_table.len() as f64 / 1024.0
  );

  println!(
    "Data values: {} ({} KB)",
    full_data_chunk.len(),
    full_data_chunk.len() as f64 / 1024.0
  );

  println!(
    "Total: {} KB",
    (id_start_table.len() + id_continue_table.len() + full_data_chunk.len()) as f64 / 1024.0
  );

  Ok(())
}

fn parse_hex(value: &str) -> u32 {
  u32::from_str_radix(value, 16).expect("Invalid character code.")
}

fn generate_charsets() -> (HashSet<u32>, HashSet<u32>) {
  let mut charset_id_start = HashSet::<u32>::new();
  let mut charset_id_continue = HashSet::<u32>::new();

  let unicode_data_file = File::open("./ucd/UnicodeData.txt").unwrap();
  let prop_list_file = File::open("./ucd/PropList.txt").unwrap();

  for line in io::BufReader::new(unicode_data_file).lines().flatten() {
    let data = line.split(';').collect::<Vec<&str>>();
    let char_code = data[0];
    let char_class = data[2];

    if matches!(char_class, "Lu" | "Ll" | "Lt" | "Lm" | "Lo" | "Nl") {
      let code = parse_hex(char_code);

      charset_id_start.insert(code);
      charset_id_continue.insert(code);
    }

    if matches!(char_class, "Mn" | "Mc" | "Nd" | "Pc") {
      let code = parse_hex(char_code);

      charset_id_continue.insert(code);
    }
  }

  for line in io::BufReader::new(prop_list_file).lines().flatten() {
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    let split_start = line.find(';').expect("Invalid data format.") + 1;
    let split_end = line.find('#').expect("Invalid data format.") - 1;
    let char_class = &line[split_start..split_end];

    if matches!(
      char_class,
      "Other_ID_Start" | "Other_ID_Continue" | "Pattern_Syntax" | "Pattern_White_Space"
    ) {
      let code_split_end = line.find(' ').expect("Invalid data format.");
      let code_str = &line[0..code_split_end];

      let mut codes: Vec<u32> = Vec::new();

      if code_str.len() == 4 {
        codes.push(parse_hex(code_str));
      } else {
        let start_code = parse_hex(&code_str[0..4]);
        let end_code = parse_hex(&code_str[6..10]);

        for i in start_code..=end_code {
          codes.push(i)
        }
      }

      for code in codes {
        if char_class == "Other_ID_Start" {
          charset_id_start.insert(code);
          charset_id_continue.insert(code);
        } else if char_class == "Other_ID_Continue" {
          charset_id_continue.insert(code);
        } else {
          charset_id_start.remove(&code);
          charset_id_continue.remove(&code);
        }
      }
    }
  }

  (charset_id_start, charset_id_continue)
}

fn gen_data_table(data_chunks: &mut Vec<(u8, Vec<u8>)>, charset: &HashSet<u32>) -> (u32, Vec<u8>) {
  let max_bound = *charset.iter().max().unwrap();
  let total_rows = (max_bound as usize).div_ceil(ROW_SIZE * 8);

  let mut id_table: Vec<u8> = Vec::new();

  for row_i in 0..total_rows {
    let mut chunk: Vec<u8> = Vec::with_capacity(ROW_SIZE);

    for byte_i in 0..ROW_SIZE {
      let mut byte: u8 = 0;

      for char_i in 0..8 {
        let c = (row_i * ROW_SIZE * 8 + byte_i * 8 + char_i) as u32;

        if charset.contains(&c) {
          byte |= 1 << char_i;
        }
      }

      chunk.push(byte);
    }

    let mut found_chunk: Option<u8> = None;

    for (i, existing_chunk) in data_chunks.iter() {
      let mut matches = true;

      for (i, e) in existing_chunk.iter().enumerate() {
        if chunk[i] != *e {
          matches = false;
          break;
        }
      }

      if matches {
        found_chunk = Some(*i);
        break;
      }
    }

    let chunk_index = match found_chunk {
      Some(index) => index,
      None => {
        let index = data_chunks.len() as u8;
        data_chunks.push((index, chunk));
        index
      }
    };

    if chunk_index > 0xEF {
      panic!("Chunk index exceeded maximum value of 0xEF");
    }

    id_table.push(chunk_index);
  }

  (max_bound, id_table)
}

fn merge_chunks(mut unmerged_chunks: Vec<(u8, Vec<u8>)>) -> (Vec<u8>, HashMap<u8, u8>) {
  let mut chunk_index_map: HashMap<u8, u8> = HashMap::new();
  let mut full_data_chunk: Vec<u8> = Vec::new();

  let mut write_index: u8 = 0;
  let mut first_chunk = unmerged_chunks.remove(0);

  chunk_index_map.insert(first_chunk.0, write_index);
  full_data_chunk.append(&mut first_chunk.1);

  'main_loop: loop {
    if unmerged_chunks.len() == 0 {
      break;
    }

    write_index += 1;

    let last_half = &full_data_chunk[(full_data_chunk.len() - ROW_SIZE / 2)..full_data_chunk.len()];

    for chunk_i in 0..unmerged_chunks.len() {
      let chunk = &unmerged_chunks[chunk_i];
      let mut matches = true;

      for (i, byte) in last_half.iter().enumerate() {
        if chunk.1[i] != *byte {
          matches = false;
          break;
        }
      }

      if matches {
        let chunk = unmerged_chunks.remove(chunk_i);

        chunk_index_map.insert(chunk.0, write_index);
        full_data_chunk.append(&mut chunk.1[(ROW_SIZE / 2)..ROW_SIZE].to_vec());

        continue 'main_loop;
      }
    }

    write_index += 1;

    let mut next_chunk = unmerged_chunks.remove(0);

    chunk_index_map.insert(next_chunk.0, write_index);
    full_data_chunk.append(&mut next_chunk.1);
  }

  (full_data_chunk, chunk_index_map)
}
