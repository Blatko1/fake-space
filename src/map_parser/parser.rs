fn parse(data: String) {
    let mut lines = data
    .lines()
    .enumerate()
    .map(|(i, line)| (i, line.split("//").next().unwrap().trim()))
    .filter(|(_, line)| !line.is_empty());
}