use std::io::Write;

use crate::*;

//the total number of vertical lines ( | ) that appear in the [-|||...|-] in the overview section
static NUM_OF_VERTICALS : usize = 50;

static KEYWORD_LINE_OFFSET : usize = 20;
static STANDARD_LINE_STATS_LEN : usize = 33;

pub fn format_and_print_results(content_info_map: &mut HashMap<String, ExtensionContentInfo>,
        extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    remove_extensions_with_0_files(content_info_map, extensions_metadata_map);

    let mut sorted_extension_names = 
            get_extension_names_as_sorted_vec_according_to_how_much_they_appeared(extensions_metadata_map);

    print_individually(&sorted_extension_names, &content_info_map, extensions_metadata_map);

    if extensions_metadata_map.len() > 1 {
        print_sum(&content_info_map, &extensions_metadata_map);
        print_visual_overview(&mut sorted_extension_names, content_info_map, extensions_metadata_map);
    }
}


fn print_individually(sorted_extensions_map: &[String], content_info_map: &HashMap<String,ExtensionContentInfo>,
     extensions_metadata_map: &HashMap<String, ExtensionMetadata>)
{
    fn get_size_text(metadata: &ExtensionMetadata) -> String {
        let (size, size_desc) = get_size_and_formatted_size_text(metadata.bytes, "total");
        let (average_size, average_size_desc) = get_size_and_formatted_size_text(
                metadata.bytes / metadata.files, "average");

        format!("{:.1} {} - {:.1} {}",size, size_desc, average_size, average_size_desc)
    }

    fn reconstruct_line(i: usize, max_line_stats_len: usize, titles_vec: &[String], lines_stats_vec: &[String],
         lines_stats_len_vec: &[usize], size_stats_vec: &[String], keywords_stats_vec: &[String]) -> String
    {
        let spaces = max_line_stats_len+1 - lines_stats_len_vec[i];
        titles_vec[i].clone() + &lines_stats_vec[i] + &" ".repeat(spaces) + " |  " + &size_stats_vec[i] +
                "\n" + &keywords_stats_vec[i]
    }

    println!("{}.\n", "Details".underline().bold());
    
    let max_files_num_size = extensions_metadata_map.values().map(|x| x.files).max().unwrap().to_string().len();
    let mut max_line_stats_len = STANDARD_LINE_STATS_LEN;
    let (mut titles_vec, mut lines_stats_vec, mut lines_stats_len_vec, mut size_stats_vec,
            mut keywords_stats_vec) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for extension_name in sorted_extensions_map {
        let content_info = content_info_map.get(extension_name).unwrap();
        let metadata = extensions_metadata_map.get(extension_name).unwrap();

        let files_str = with_seperators(metadata.files);
        let title = format!(".{}{}{}{} {}  -> ",extension_name.bold(), " ".repeat(7-extension_name.len()),
                " ".repeat((max_files_num_size+1)-files_str.len()), files_str, colored_word("files"));
        titles_vec.push(title);

        let code_lines_percentage = if content_info.lines > 0 {content_info.code_lines as f64 / content_info.lines as f64 * 100f64} else {0f64};
        let lines_str = with_seperators(content_info.lines);
        let code_lines_str = with_seperators(content_info.code_lines);
        let extra_lines_str = with_seperators(content_info.lines - content_info.code_lines);
        let curr_line_stats_len = STANDARD_LINE_STATS_LEN + lines_str.len() + code_lines_str.len() + extra_lines_str.len();
        lines_stats_len_vec.push(curr_line_stats_len); 
        if max_line_stats_len < curr_line_stats_len {
            max_line_stats_len = curr_line_stats_len;
        }
        
        lines_stats_vec.push(format!("{} {} {{{} code ({:.2}%) + {} extra}}", colored_word("lines"), lines_str, code_lines_str,
                 code_lines_percentage, extra_lines_str));
        size_stats_vec.push(get_size_text(metadata));
        
        keywords_stats_vec.push(get_keywords_as_str(&content_info.keyword_occurences, max_files_num_size));
    }

    for i in 0..lines_stats_vec.len() {
        let line = reconstruct_line(i, max_line_stats_len, &titles_vec, &lines_stats_vec,
                &lines_stats_len_vec, &size_stats_vec, &keywords_stats_vec);
                
        if i == lines_stats_len_vec.len() - 1 {
            println!("{}",line);
        } else {
            println!("{}\n",line);
        }
    }
}


fn print_sum(content_info_map: &HashMap<String,ExtensionContentInfo>, extensions_metadata_map: &HashMap<String,ExtensionMetadata>) 
{
    let (mut total_files, mut total_lines, mut total_code_lines, mut total_bytes) = (0, 0, 0,0);
    extensions_metadata_map.values().for_each(|e| {total_files += e.files; total_bytes += e.bytes});
    content_info_map.values().for_each(|c| {total_lines += c.lines; total_code_lines += c.code_lines});

    let (total_files_str, total_lines_str, total_code_lines_str, total_extra_lines_str) = 
            (with_seperators(total_files),with_seperators(total_lines),with_seperators(total_code_lines), with_seperators(total_lines-total_code_lines)); 
    let (total_size, total_size_descr) = get_size_and_formatted_size_text(total_bytes, "total");
    let (average_size, average_size_descr) = get_size_and_formatted_size_text(total_bytes / total_files, "average");

    let max_files_num_size = extensions_metadata_map.values().map(|x| x.files).max().unwrap().to_string().len();

    let keywords_sum_map = create_keyword_sum_map(content_info_map);
    let keywords_line = get_keywords_as_str(&keywords_sum_map, max_files_num_size);

    let title = format!("{}   {}{} {}  -> ","total".bold()," ".repeat(max_files_num_size+1 -total_files_str.len()),total_files_str,colored_word("files"));
    let code_lines_percentage = if total_lines > 0 {total_code_lines as f64 / total_lines as f64 * 100f64} else {0f64};
    let size_text = format!("{:.1} {} - {:.1} {}",total_size,total_size_descr,average_size,average_size_descr);

    let line_len = STANDARD_LINE_STATS_LEN + total_files_str.len() + total_code_lines_str.len() + total_extra_lines_str.len() +
            total_size.to_string().len() + average_size.to_string().len() + 47;
    println!("{} ","-".repeat(line_len));
    
    let info = format!("{} {} {{{} code ({:.2}%) + {} extra}}  |  {}\n",colored_word("lines"), total_lines_str,total_code_lines_str,
            code_lines_percentage, total_extra_lines_str, size_text);

    println!("{}", format!("{}{}{}\n",title,info,keywords_line));
}

//                                    OVERVIEW
//
// Files:    47% java - 32% cs - 21% py        [-||||||||||||||||||||||||||||||||||||||||||||||||||] 
//
// Lines: ...
//
// Size : ...
fn print_visual_overview(sorted_extension_vec: &mut Vec<String>, content_info_map: &mut HashMap<String, ExtensionContentInfo>,
     extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    fn make_cyan(str: &str) -> String {
        str.cyan().to_string()
    }
    fn make_magenta(str: &str) -> String {
        str.bright_magenta().to_string()
    }
    fn make_yellow(str: &str) -> String {
        str.bright_yellow().to_string()
    }
    fn no_transformation(str: &str) -> String {
        str.to_owned()
    }
    fn make_fourth_color(str: &str) -> String {
        str.truecolor(106, 217, 189).to_string()
    }
    fn make_color_for_others(str: &str) -> String {
        str.truecolor(215, 201, 240).to_string()
    }

    if content_info_map.len() > 4 {
        retain_most_relevant_and_add_others_field_for_rest(sorted_extension_vec, content_info_map, extensions_metadata_map);
    }

    println!("{}.\n", "Overview".underline().bold());

    let color_func_vec : Vec<fn(&str) -> String> = {
        if sorted_extension_vec[sorted_extension_vec.len()-1] == "others" {
            vec![make_cyan, make_magenta, make_yellow, make_color_for_others]
        } else {
            vec![make_cyan, make_magenta, make_yellow, make_fourth_color]
        }
    };

    let files_percentages = get_files_percentages(extensions_metadata_map, sorted_extension_vec);
    let lines_percentages = get_lines_percentages(content_info_map, sorted_extension_vec);
    let sizes_percentages = get_sizes_percentages(extensions_metadata_map, sorted_extension_vec);

    let files_verticals = get_num_of_verticals(&files_percentages);
    let lines_verticals = get_num_of_verticals(&lines_percentages);
    let size_verticals = get_num_of_verticals(&sizes_percentages);

    let files_line = create_overview_line("Files:", &files_percentages, &files_verticals, sorted_extension_vec, &color_func_vec);
    let lines_line = create_overview_line("Lines:", &lines_percentages, &lines_verticals, sorted_extension_vec, &color_func_vec);
    let size_line = create_overview_line("Size :", &sizes_percentages, &size_verticals, sorted_extension_vec, &color_func_vec);

    println!("{}\n\n{}\n\n{}\n",files_line, lines_line, size_line);
}


fn get_keywords_as_str(keyword_occurencies: &HashMap<String,usize>, max_files_num_size: usize) -> String {
    let mut keyword_info = String::new();
    if !keyword_occurencies.is_empty() {
        let mut keyword_iter = keyword_occurencies.iter();
        let first_keyword = keyword_iter.next().unwrap();
        keyword_info.push_str(&format!("{}{}: {}"," ".repeat(KEYWORD_LINE_OFFSET + max_files_num_size),
                colored_word(first_keyword.0),with_seperators(*first_keyword.1)));
        for (keyword_name,occurancies) in keyword_iter {
            keyword_info.push_str(&format!(" , {}: {}",colored_word(keyword_name),with_seperators(*occurancies)));
        }
    }
    keyword_info
}

fn create_keyword_sum_map(content_info_map: &HashMap<String,ExtensionContentInfo>) -> HashMap<String,usize> {
    let mut collective_keywords_map : HashMap<String,usize> = HashMap::new();
    for content_info in content_info_map.values() {
        for keyword in &content_info.keyword_occurences {
            if *keyword.1 == 0 {continue;}
            if let Some(x) = collective_keywords_map.get_mut(keyword.0) {
                *x += *keyword.1;
            } else {
                collective_keywords_map.insert(keyword.0.to_owned(), *keyword.1);
            }
        }
    }

    collective_keywords_map
}

fn get_size_and_formatted_size_text(value: usize, suffix: &str) -> (f64,ColoredString) {
    if value > 1000000 
        {(value as f64 / 1000000f64, colored_word(&("MBs ".to_owned() + suffix)))}
    else if value > 1000 
        {(value as f64 / 1000f64, colored_word(&("KBs ".to_owned() + suffix)))}
    else
        {(value as f64, colored_word(&("Bytes ".to_owned() + suffix)))}
}

fn colored_word(word: &str) -> ColoredString {
    word.italic().truecolor(181, 169, 138)
}

fn remove_extensions_with_0_files(content_info_map: &mut HashMap<String,ExtensionContentInfo>,
    extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
   let mut empty_extensions = Vec::new();
   for element in extensions_metadata_map.iter() {
       if element.1.files == 0 {
           empty_extensions.push(element.0.to_owned());
       }
   }

   for ext in empty_extensions {
       extensions_metadata_map.remove(&ext);
       content_info_map.remove(&ext);
   }
}

fn get_extension_names_as_sorted_vec_according_to_how_much_they_appeared(
   extensions_metadata_map: &HashMap<String, ExtensionMetadata>) -> Vec<String> 
{
    let mut value_map = HashMap::<String,usize>::new();
    let mut sorted_extensions_vec = Vec::new();
    for (ext_name,metadata) in extensions_metadata_map.iter() {
        value_map.insert(ext_name.to_owned(), metadata.files * 10 + metadata.bytes as usize);
        sorted_extensions_vec.push(ext_name.to_owned());
    }

    sorted_extensions_vec.sort_by(|a,b| {
        value_map.get(b).unwrap().cmp(value_map.get(a).unwrap())
    });

    sorted_extensions_vec
}

fn get_num_of_verticals(percentages: &[f64]) -> Vec<usize> {
    let mut verticals = Vec::<usize>::with_capacity(4);
    let mut sum = 0;
    for files_percent in percentages.iter(){
        let num_of_verticals = 
        if *files_percent == 0f64 {
            0
        } else {
            let mut num_of_verticals = (files_percent/2.0).round() as usize;
            if num_of_verticals == 0 {
                num_of_verticals = 1;
            }
            num_of_verticals
        };
        verticals.push(num_of_verticals);
        sum += num_of_verticals;
    }

    if sum != NUM_OF_VERTICALS {
        normalize_to_NUM_OF_VERTICALS(&mut verticals, sum);
    }

    verticals
}

// A not very precise attempt to normalize the sum of verticals to the proper number that should appear 
// in the [-|||...|-] block, but is it good enough.
fn normalize_to_NUM_OF_VERTICALS(verticals: &mut Vec<usize>, sum: usize) {
    let mut sorted_verticals = Vec::new();
    for i in verticals.iter_mut() {
        sorted_verticals.push(i);
    }

    let comparator = |a: &&mut usize,b: &&mut usize| b.cmp(a);
    sorted_verticals.sort_by(comparator);

    let is_over = sum > NUM_OF_VERTICALS;
    let mut difference = if is_over {sum - NUM_OF_VERTICALS} else {NUM_OF_VERTICALS - sum}; 

    let same_num_of_verticals_indices = {
        let mut temp = Vec::new();
        let max_value = *sorted_verticals[0];
        let mut counter = 0;
        while counter < sorted_verticals.len() && *sorted_verticals[counter] == max_value {
            temp.push(counter);
            counter += 1;
        }
        temp
    };

    //ensures that if there are very close percentages, they wont have more than one vertical difference
    if same_num_of_verticals_indices.len() > 1 {
        for i in same_num_of_verticals_indices.iter() {
            if difference > 0 {
                if is_over {
                    *sorted_verticals[*i] -= 1
                } else {
                    *sorted_verticals[*i] += 1;
                }
                difference -= 1;
            } else {
                break;
            }
        }
    }

    if difference == 0 {return;}

    if is_over {
        *sorted_verticals[0] -= 1; 
        sorted_verticals.sort_by(comparator);
    } else {
        *sorted_verticals[0] += 1;
    }
    
    for _ in 0..difference-1 {
        if is_over {
            if *sorted_verticals[0] > *sorted_verticals[1] + 3 {
                *sorted_verticals[0] -= 1;
            } else {
                *sorted_verticals[1] -= 1;
                if sorted_verticals.len() > 2 {
                    sorted_verticals.sort_by(comparator);

                }
            }
        } else {
            if *sorted_verticals[0] > *sorted_verticals[1] + 5 {
                *sorted_verticals[1] += 1;
                if sorted_verticals.len() > 2 {
                    sorted_verticals.sort_by(comparator);
                }
            } else {
                *sorted_verticals[0] += 1;
            }
        }
    }
}

fn create_overview_line(prefix: &str, percentages: &[f64], verticals: &[usize],
        extensions_name: &[String], color_func_vec: &[fn(&str) -> String]) -> String 
{
    let mut line = String::with_capacity(150);
    line.push_str(&format!("{}    ",prefix));
    for (i,percent) in percentages.iter().enumerate() {
        let str_perc = format!("{:.1}",percent);
        line.push_str(&format!("{}{}% ", " ".repeat(4-str_perc.len()), str_perc));
        line.push_str(&color_func_vec[i](&extensions_name[i]));
        if i < percentages.len() - 1{
            line.push_str(" - ")
        }
    }
    
    add_verticals_str(&mut line, verticals, color_func_vec);

    line
}

fn add_verticals_str(line: &mut String, files_verticals: &[usize], color_func_vec: &[fn(&str) -> String]) {
    line.push_str("    [-");
    for (i,verticals) in files_verticals.iter().enumerate() {
        line.push_str(&color_func_vec[i]("|").repeat(*verticals));
    }
    line.push_str("-]");
}

fn retain_most_relevant_and_add_others_field_for_rest(sorted_extension_names: &mut Vec<String>,
     content_info_map: &mut HashMap<String, ExtensionContentInfo>, extensions_metadata_map: &mut HashMap<String, ExtensionMetadata>) 
{
    fn get_files_lines_size(content_info_map: &HashMap<String, ExtensionContentInfo>,
         extensions_metadata_map: &HashMap<String, ExtensionMetadata>) -> (usize,usize,usize) 
    {
        let (mut files, mut lines, mut size) = (0,0,0);
        content_info_map.iter().for_each(|x| lines += x.1.lines);
        extensions_metadata_map.iter().for_each(|x| {files += x.1.files; size += x.1.bytes});
        (files, lines, size as usize) 
    }

    let (total_files, total_lines, total_size) = get_files_lines_size(content_info_map, extensions_metadata_map);
    if sorted_extension_names.len() > 4 {
        for _ in 3..sorted_extension_names.len() {
             sorted_extension_names.remove(sorted_extension_names.len()-1);
        }
        sorted_extension_names.push("others".to_owned());

        content_info_map.retain(|x,_| sorted_extension_names.contains(x));
        extensions_metadata_map.retain(|x,_| sorted_extension_names.contains(x));
    }
    
    let (relevant_files, relevant_lines, relevant_size) = get_files_lines_size(content_info_map, extensions_metadata_map);
    let (other_files, other_lines, other_size) = 
        (total_files - relevant_files, total_lines - relevant_lines, total_size - relevant_size);

    content_info_map.insert("others".to_string(), ExtensionContentInfo::dummy(other_lines));
    extensions_metadata_map.insert("others".to_string(), ExtensionMetadata::new(other_files, other_size));
}


fn get_files_percentages(extensions_metadata_map: &HashMap<String,ExtensionMetadata>, sorted_extension_names: &[String]) -> Vec<f64> {
    let mut extensions_files = [0].repeat(extensions_metadata_map.len());
    extensions_metadata_map.iter().for_each(|e| {
        let pos = sorted_extension_names.iter().position(|name| name == e.0).unwrap();
        extensions_files[pos] = e.1.files;
    });
    
    get_percentages(&extensions_files)
}

fn get_lines_percentages(content_info_map: &HashMap<String,ExtensionContentInfo>, extensions_name: &[String]) -> Vec<f64> {
    let mut extensions_lines = [0].repeat(content_info_map.len());
    content_info_map.iter().for_each(|e| {
        let pos = extensions_name.iter().position(|name| name == e.0).unwrap();
        extensions_lines[pos] = e.1.lines;
    });

    get_percentages(&extensions_lines)
}

fn get_sizes_percentages(extensions_metadata_map: &HashMap<String,ExtensionMetadata>, extensions_name: &[String]) -> Vec<f64> {
    let mut extensions_size = [0].repeat(extensions_metadata_map.len());
    extensions_metadata_map.iter().for_each(|e| {
        let pos = extensions_name.iter().position(|name| name == e.0).unwrap();
        extensions_size[pos] = e.1.bytes;
    });
    
    get_percentages(&extensions_size)
}

fn get_percentages(numbers: &[usize]) -> Vec<f64> {
    let total_files :usize = numbers.iter().sum();
    let mut extension_percentages = Vec::with_capacity(4);
    let mut sum = 0.0;
    for (counter,files) in numbers.iter().enumerate() {
        if counter == numbers.len() - 1 {
            let rounded = {
                if sum > 99.89 {
                    0.0
                } else {
                    ((100f64 - sum) * 10f64).round() / 10f64
                }
            };
            extension_percentages.push(rounded);
        } else {
            let percentage = *files as f64/total_files as f64;
            let canonicalized = (percentage * 1000f64).round() / 10f64;
            sum += canonicalized;
            extension_percentages.push(canonicalized);
        }
    }
    extension_percentages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let mut verticals = vec![18,15,19,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 53);
        assert_eq!(vec![16,15,18,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
        
        let mut verticals = vec![17,17,18,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 53);
        assert_eq!(vec![16,16,17,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![16,15,16,1];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 48);
        assert_eq!(vec![17,15,17,1], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![18,16,17];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 51);
        assert_eq!(vec![17,16,17], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    
        let mut verticals = vec![25,26];
        normalize_to_NUM_OF_VERTICALS(&mut verticals, 51);
        assert_eq!(vec![25,25], verticals);
        assert!(verticals.iter().sum::<usize>() == 50);
    }

    #[test]
    fn test_get_lines_percentages() {
        let ext_names = ["py".to_string(),"java".to_string(),"cs".to_string()];

        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(100),
            "java".to_string() => ExtensionContentInfo::dummy(100), "py".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![0f64,50f64,50f64], get_lines_percentages(&content_info_map, &ext_names));
        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(0),
        "java".to_string() => ExtensionContentInfo::dummy(0), "py".to_string() => ExtensionContentInfo::dummy(1));
        assert_eq!(vec![100f64,0f64,0f64], get_lines_percentages(&content_info_map, &ext_names));
        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(20),
        "java".to_string() => ExtensionContentInfo::dummy(20), "py".to_string() => ExtensionContentInfo::dummy(20));
        assert_eq!(vec![33.3f64,33.3f64,33.4f64], get_lines_percentages(&content_info_map, &ext_names));
        
        let ext_names = ["py".to_string(),"java".to_string(),"cs".to_string(),"rs".to_string()];

        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(100),
            "java".to_string() => ExtensionContentInfo::dummy(100), "py".to_string() => ExtensionContentInfo::dummy(0),
            "rs".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![0f64,50f64,50f64,0f64], get_lines_percentages(&content_info_map, &ext_names));
        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(100),
            "java".to_string() => ExtensionContentInfo::dummy(100), "py".to_string() => ExtensionContentInfo::dummy(100),
            "rs".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![33.3f64,33.3f64,33.3f64,0f64], get_lines_percentages(&content_info_map, &ext_names));
        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(201),
            "java".to_string() => ExtensionContentInfo::dummy(200), "py".to_string() => ExtensionContentInfo::dummy(200),
            "rs".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![33.3f64,33.3f64,33.4f64,0f64], get_lines_percentages(&content_info_map, &ext_names));

        let ext_names = ["py".to_string(),"java".to_string(),"cs".to_string(),"rs".to_string(),"cpp".to_string()];

        let content_info_map = hashmap!("cs".to_string() => ExtensionContentInfo::dummy(100),
            "java".to_string() => ExtensionContentInfo::dummy(100), "py".to_string() => ExtensionContentInfo::dummy(0),
            "rs".to_string() => ExtensionContentInfo::dummy(0), "cpp".to_string() => ExtensionContentInfo::dummy(0));
        assert_eq!(vec![0f64,50f64,50f64,0f64,0f64], get_lines_percentages(&content_info_map, &ext_names));
    }

    #[test]
    fn test_get_num_of_verticals() {
        let percentages = vec![49.6,50.4];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![25,25], verticals);

        let percentages = vec![0.0,100.0];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![0,50], verticals);


        let percentages = vec![33.33,33.33,33.34];
        assert_eq!(vec![16,17,17], get_num_of_verticals(&percentages));

        let percentages = vec![0.3,65.67,34.3];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![1,32,17], verticals);
        
        let percentages = vec![0.0,0.0,100.0];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![0,0,50], verticals);

        let percentages = vec![0.2,49.9,49.9];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![1,24,25], verticals);


        let percentages = vec![12.5,50.0,25.0,12.5];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![6,25,13,6], verticals);

        let percentages = vec![0.1,0.1,49.9,49.9];
        let verticals = get_num_of_verticals(&percentages);
        assert!(verticals.iter().sum::<usize>() == 50);
        assert_eq!(vec![1,1,24,24], verticals);
    }
}