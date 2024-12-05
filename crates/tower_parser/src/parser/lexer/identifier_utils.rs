use unicode_id::{
  DATA_TABLE, ID_CONTINUE_TABLE, ID_START_TABLE, MAX_BOUND_ID_CONTINUE, MAX_BOUND_ID_START,
  ROW_SIZE,
};

pub fn is_id_start(c: char) -> bool {
  if c == '$' || c == '_' {
    return true;
  }

  if (c as u32) > MAX_BOUND_ID_START {
    return false;
  }

  unsafe {
    let chunk_index = *ID_START_TABLE.get_unchecked(c as usize / ROW_SIZE / 8);
    let data_byte =
      *DATA_TABLE.get_unchecked(chunk_index as usize * ROW_SIZE / 2 + c as usize / 8 % ROW_SIZE);

    ((data_byte >> ((c as u32) % 8)) & 1) == 1
  }
}

pub fn is_id_continue(c: char) -> bool {
  if c == '$' {
    return true;
  }

  if (c as u32) > MAX_BOUND_ID_CONTINUE {
    return false;
  }

  unsafe {
    let chunk_index = *ID_CONTINUE_TABLE.get_unchecked(c as usize / ROW_SIZE / 8);
    let data_byte =
      *DATA_TABLE.get_unchecked(chunk_index as usize * ROW_SIZE / 2 + c as usize / 8 % ROW_SIZE);

    ((data_byte >> ((c as u32) % 8)) & 1) == 1
  }
}
