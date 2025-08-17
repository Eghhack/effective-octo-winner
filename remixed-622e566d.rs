// Organizador Semanal em Rust
// Versão: 1.0.0
// Autor: Claude AI
// Descrição: Sistema de organização semanal com blocos de 30 minutos

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, NaiveTime, Weekday};
use uuid::Uuid;

// Estruturas de dados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub title: String,
    pub category: String,
    pub duration: f32, // Em horas (0.5 = 30 min)
    pub start_time: String, // Formato "HH:MM"
    pub location: Option<String>,
    pub description: Option<String>,
    pub day: String,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub total_time: f32,
    pub by_category: HashMap<String, f32>,
    pub by_day: HashMap<String, f32>,
    pub activity_count: usize,
}

pub struct WeeklyOrganizer {
    activities: Vec<Activity>,
    categories: HashMap<String, Category>,
    data_file: String,
}

impl WeeklyOrganizer {
    // Construtor
    pub fn new(data_file: &str) -> Self {
        let mut organizer = WeeklyOrganizer {
            activities: Vec::new(),
            categories: HashMap::new(),
            data_file: data_file.to_string(),
        };
        
        // Categorias padrão
        organizer.init_default_categories();
        
        // Carregar dados existentes
        if let Err(e) = organizer.load_data() {
            println!("Aviso: Não foi possível carregar dados existentes: {}", e);
            println!("Iniciando com dados limpos.");
        }
        
        organizer
    }
    
    // Inicializar categorias padrão
    fn init_default_categories(&mut self) {
        let default_categories = [
            ("trabalho", Category { name: "Trabalho".to_string(), color: "#3B82F6".to_string() }),
            ("pessoal", Category { name: "Pessoal".to_string(), color: "#10B981".to_string() }),
            ("saude", Category { name: "Saúde".to_string(), color: "#F59E0B".to_string() }),
            ("estudo", Category { name: "Estudo".to_string(), color: "#8B5CF6".to_string() }),
            ("lazer", Category { name: "Lazer".to_string(), color: "#EF4444".to_string() }),
            ("reuniao", Category { name: "Reunião".to_string(), color: "#F97316".to_string() }),
            ("exercicio", Category { name: "Exercício".to_string(), color: "#06B6D4".to_string() }),
        ];
        
        for (key, category) in default_categories {
            self.categories.insert(key.to_string(), category);
        }
    }
    
    // Gerar horários de 30 em 30 minutos
    pub fn generate_time_slots() -> Vec<String> {
        let mut slots = Vec::new();
        for hour in 6..23 { // 6h às 22h30
            slots.push(format!("{:02}:00", hour));
            slots.push(format!("{:02}:30", hour));
        }
        slots
    }
    
    // Validar horário
    fn validate_time(&self, time: &str) -> Result<(), String> {
        if NaiveTime::parse_from_str(time, "%H:%M").is_err() {
            return Err(format!("Horário inválido: {}", time));
        }
        Ok(())
    }
    
    // Validar dia da semana
    fn validate_day(&self, day: &str) -> Result<(), String> {
        let valid_days = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];
        if !valid_days.contains(&day) {
            return Err(format!("Dia inválido: {}. Use: {}", day, valid_days.join(", ")));
        }
        Ok(())
    }
    
    // Verificar conflito de horários
    fn check_time_conflict(&self, day: &str, start_time: &str, duration: f32) -> Option<&Activity> {
        let start = NaiveTime::parse_from_str(start_time, "%H:%M").unwrap();
        let end_minutes = start.hour() as i32 * 60 + start.minute() as i32 + (duration * 60.0) as i32;
        let end_hour = end_minutes / 60;
        let end_min = end_minutes % 60;
        
        for activity in &self.activities {
            if activity.day == day {
                let activity_start = NaiveTime::parse_from_str(&activity.start_time, "%H:%M").unwrap();
                let activity_end_minutes = activity_start.hour() as i32 * 60 + activity_start.minute() as i32 + (activity.duration * 60.0) as i32;
                
                let start_minutes = start.hour() as i32 * 60 + start.minute() as i32;
                
                // Verificar sobreposição
                if (start_minutes < activity_end_minutes) && (end_minutes > activity_start.hour() as i32 * 60 + activity_start.minute() as i32) {
                    return Some(activity);
                }
            }
        }
        None
    }
    
    // Adicionar nova atividade
    pub fn add_activity(&mut self, title: &str, category: &str, day: &str, start_time: &str, duration: f32, location: Option<String>, description: Option<String>) -> Result<String, String> {
        // Validações
        self.validate_day(day)?;
        self.validate_time(start_time)?;
        
        if !self.categories.contains_key(category) {
            return Err(format!("Categoria '{}' não existe", category));
        }
        
        if duration <= 0.0 || duration > 8.0 {
            return Err("Duração deve ser entre 0.5 e 8 horas".to_string());
        }
        
        if title.trim().is_empty() {
            return Err("Título não pode estar vazio".to_string());
        }
        
        // Verificar conflitos
        if let Some(conflicting_activity) = self.check_time_conflict(day, start_time, duration) {
            return Err(format!("Conflito de horário com: '{}'", conflicting_activity.title));
        }
        
        // Criar atividade
        let activity = Activity {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            category: category.to_string(),
            duration,
            start_time: start_time.to_string(),
            location,
            description,
            day: day.to_string(),
            created_at: Local::now(),
        };
        
        let id = activity.id.clone();
        self.activities.push(activity);
        
        // Salvar automaticamente
        if let Err(e) = self.save_data() {
            println!("Aviso: Erro ao salvar dados: {}", e);
        }
        
        Ok(id)
    }
    
    // Editar atividade
    pub fn edit_activity(&mut self, id: &str, title: Option<&str>, category: Option<&str>, day: Option<&str>, start_time: Option<&str>, duration: Option<f32>, location: Option<String>, description: Option<String>) -> Result<(), String> {
        let activity = self.activities.iter_mut().find(|a| a.id == id)
            .ok_or("Atividade não encontrada")?;
        
        // Criar uma cópia para validação
        let mut temp_activity = activity.clone();
        
        // Aplicar mudanças temporariamente
        if let Some(t) = title { temp_activity.title = t.to_string(); }
        if let Some(c) = category { temp_activity.category = c.to_string(); }
        if let Some(d) = day { temp_activity.day = d.to_string(); }
        if let Some(st) = start_time { temp_activity.start_time = st.to_string(); }
        if let Some(dur) = duration { temp_activity.duration = dur; }
        
        // Validações
        self.validate_day(&temp_activity.day)?;
        self.validate_time(&temp_activity.start_time)?;
        
        if !self.categories.contains_key(&temp_activity.category) {
            return Err(format!("Categoria '{}' não existe", temp_activity.category));
        }
        
        // Verificar conflitos (excluindo a própria atividade)
        let original_id = activity.id.clone();
        let activities_without_current: Vec<_> = self.activities.iter().filter(|a| a.id != original_id).cloned().collect();
        let temp_organizer = WeeklyOrganizer {
            activities: activities_without_current,
            categories: self.categories.clone(),
            data_file: self.data_file.clone(),
        };
        
        if let Some(conflicting) = temp_organizer.check_time_conflict(&temp_activity.day, &temp_activity.start_time, temp_activity.duration) {
            return Err(format!("Conflito de horário com: '{}'", conflicting.title));
        }
        
        // Aplicar mudanças
        if let Some(t) = title { activity.title = t.to_string(); }
        if let Some(c) = category { activity.category = c.to_string(); }
        if let Some(d) = day { activity.day = d.to_string(); }
        if let Some(st) = start_time { activity.start_time = st.to_string(); }
        if let Some(dur) = duration { activity.duration = dur; }
        if let Some(loc) = location { activity.location = Some(loc); }
        if let Some(desc) = description { activity.description = Some(desc); }
        
        // Salvar
        if let Err(e) = self.save_data() {
            println!("Aviso: Erro ao salvar dados: {}", e);
        }
        
        Ok(())
    }
    
    // Remover atividade
    pub fn remove_activity(&mut self, id: &str) -> Result<(), String> {
        let initial_len = self.activities.len();
        self.activities.retain(|a| a.id != id);
        
        if self.activities.len() == initial_len {
            return Err("Atividade não encontrada".to_string());
        }
        
        // Salvar
        if let Err(e) = self.save_data() {
            println!("Aviso: Erro ao salvar dados: {}", e);
        }
        
        Ok(())
    }
    
    // Listar atividades de um dia
    pub fn get_activities_by_day(&self, day: &str) -> Vec<&Activity> {
        let mut activities: Vec<&Activity> = self.activities.iter()
            .filter(|a| a.day == day)
            .collect();
        
        // Ordenar por horário
        activities.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        activities
    }
    
    // Obter todas as atividades ordenadas
    pub fn get_all_activities(&self) -> Vec<&Activity> {
        let mut activities: Vec<&Activity> = self.activities.iter().collect();
        activities.sort_by(|a, b| {
            let day_order = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];
            let a_day_idx = day_order.iter().position(|&d| d == a.day).unwrap_or(7);
            let b_day_idx = day_order.iter().position(|&d| d == b.day).unwrap_or(7);
            
            a_day_idx.cmp(&b_day_idx)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });
        activities
    }
    
    // Calcular estatísticas semanais
    pub fn calculate_weekly_stats(&self) -> WeeklyStats {
        let mut stats = WeeklyStats {
            total_time: 0.0,
            by_category: HashMap::new(),
            by_day: HashMap::new(),
            activity_count: self.activities.len(),
        };
        
        for activity in &self.activities {
            // Tempo total
            stats.total_time += activity.duration;
            
            // Por categoria
            *stats.by_category.entry(activity.category.clone()).or_insert(0.0) += activity.duration;
            
            // Por dia
            *stats.by_day.entry(activity.day.clone()).or_insert(0.0) += activity.duration;
        }
        
        stats
    }
    
    // Formatar tempo
    pub fn format_time(hours: f32) -> String {
        if hours < 1.0 {
            format!("{}min", (hours * 60.0).round() as i32)
        } else if hours == 1.0 {
            "1h".to_string()
        } else if hours.fract() == 0.0 {
            format!("{}h", hours as i32)
        } else {
            let whole_hours = hours.floor() as i32;
            let minutes = ((hours - whole_hours as f32) * 60.0).round() as i32;
            format!("{}h {}min", whole_hours, minutes)
        }
    }
    
    // Exibir grade semanal
    pub fn display_weekly_grid(&self) {
        let days = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];
        let time_slots = Self::generate_time_slots();
        
        println!("\n╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                                              ORGANIZADOR SEMANAL                                                     ║");
        println!("╠═══════════╦══════════════╦══════════════╦══════════════╦══════════════╦══════════════╦══════════════╦══════════════╣");
        print!("║   HORÁRIO ║");
        for day in &days {
            print!(" {:^12} ║", day);
        }
        println!();
        println!("╠═══════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╣");
        
        for (i, time) in time_slots.iter().enumerate() {
            // Mostrar apenas horários completos
            if time.ends_with(":00") {
                print!("║ {:^9} ║", time);
            } else {
                print!("║ {:^9} ║", "");
            }
            
            for day in &days {
                let activity = self.activities.iter().find(|a| {
                    a.day == *day && a.start_time == *time
                });
                
                match activity {
                    Some(act) => {
                        let short_title = if act.title.len() > 12 {
                            format!("{}...", &act.title[..9])
                        } else {
                            act.title.clone()
                        };
                        print!(" {:^12} ║", short_title);
                    },
                    None => print!(" {:^12} ║", ""),
                }
            }
            println!();
            
            // Linha separadora a cada hora
            if time.ends_with(":30") {
                if i < time_slots.len() - 1 {
                    println!("╠═══════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╬══════════════╣");
                }
            }
        }
        
        println!("╚═══════════╩══════════════╩══════════════╩══════════════╩══════════════╩══════════════╩══════════════╩══════════════╝");
    }
    
    // Exibir estatísticas
    pub fn display_stats(&self) {
        let stats = self.calculate_weekly_stats();
        
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║                      ESTATÍSTICAS SEMANAIS                      ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Total de atividades: {:^42} ║", stats.activity_count);
        println!("║ Tempo total semanal: {:^42} ║", Self::format_time(stats.total_time));
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║                      POR CATEGORIA                              ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        
        let mut category_stats: Vec<_> = stats.by_category.iter().collect();
        category_stats.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (category_key, time) in category_stats {
            if let Some(category) = self.categories.get(category_key) {
                let percentage = if stats.total_time > 0.0 {
                    (time / stats.total_time) * 100.0
                } else {
                    0.0
                };
                println!("║ {:20} │ {:>12} │ {:>6.1}% ║", 
                    category.name, 
                    Self::format_time(*time),
                    percentage
                );
            }
        }
        
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║                        POR DIA                                  ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        
        let days = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];
        for day in &days {
            let day_time = stats.by_day.get(*day).unwrap_or(&0.0);
            let percentage = if stats.total_time > 0.0 {
                (day_time / stats.total_time) * 100.0
            } else {
                0.0
            };
            println!("║ {:20} │ {:>12} │ {:>6.1}% ║", 
                day, 
                Self::format_time(*day_time),
                percentage
            );
        }
        
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
    
    // Salvar dados em arquivo JSON
    pub fn save_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct SaveData {
            activities: Vec<Activity>,
            categories: HashMap<String, Category>,
        }
        
        let data = SaveData {
            activities: self.activities.clone(),
            categories: self.categories.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&self.data_file, json)?;
        Ok(())
    }
    
    // Carregar dados do arquivo JSON
    pub fn load_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct SaveData {
            activities: Vec<Activity>,
            categories: HashMap<String, Category>,
        }
        
        let content = fs::read_to_string(&self.data_file)?;
        let data: SaveData = serde_json::from_str(&content)?;
        
        self.activities = data.activities;
        self.categories.extend(data.categories);
        
        Ok(())
    }
    
    // Exportar para CSV
    pub fn export_to_csv(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        content.push_str("ID,Título,Categoria,Dia,Horário,Duração(h),Local,Descrição,Criado em\n");
        
        for activity in &self.activities {
            content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                activity.id,
                activity.title.replace(",", ";"),
                activity.category,
                activity.day,
                activity.start_time,
                activity.duration,
                activity.location.as_ref().unwrap_or(&"".to_string()).replace(",", ";"),
                activity.description.as_ref().unwrap_or(&"".to_string()).replace(",", ";"),
                activity.created_at.format("%Y-%m-%d %H:%M:%S")
            ));
        }
        
        fs::write(filename, content)?;
        Ok(())
    }
    
    // Buscar atividades
    pub fn search_activities(&self, query: &str) -> Vec<&Activity> {
        let query_lower = query.to_lowercase();
        self.activities.iter()
            .filter(|activity| {
                activity.title.to_lowercase().contains(&query_lower) ||
                activity.category.to_lowercase().contains(&query_lower) ||
                activity.location.as_ref().map_or(false, |loc| loc.to_lowercase().contains(&query_lower)) ||
                activity.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

// Interface de linha de comando
pub struct CLI {
    organizer: WeeklyOrganizer,
}

impl CLI {
    pub fn new(data_file: &str) -> Self {
        CLI {
            organizer: WeeklyOrganizer::new(data_file),
        }
    }
    
    pub fn run(&mut self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                    ORGANIZADOR SEMANAL v1.0                     ║");
        println!("║                     Sistema em Rust                             ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        
        loop {
            self.show_menu();
            let choice = self.get_user_input("Escolha uma opção: ");
            
            match choice.trim() {
                "1" => self.add_activity_interactive(),
                "2" => self.list_activities(),
                "3" => self.edit_activity_interactive(),
                "4" => self.remove_activity_interactive(),
                "5" => self.organizer.display_weekly_grid(),
                "6" => self.organizer.display_stats(),
                "7" => self.search_activities_interactive(),
                "8" => self.export_csv_interactive(),
                "9" => self.list_categories(),
                "0" => {
                    println!("Salvando dados...");
                    if let Err(e) = self.organizer.save_data() {
                        println!("Erro ao salvar: {}", e);
                    } else {
                        println!("Dados salvos com sucesso!");
                    }
                    println!("Até logo! 👋");
                    break;
                }
                _ => println!("Opção inválida! Tente novamente."),
            }
        }
    }
    
    fn show_menu(&self) {
        println!("\n┌──────────────────────────────────────────────────────────────────┐");
        println!("│                           MENU PRINCIPAL                        │");
        println!("├──────────────────────────────────────────────────────────────────┤");
        println!("│  1. Adicionar atividade                                         │");
        println!("│  2. Listar atividades                                           │");
        println!("│  3. Editar atividade                                            │");
        println!("│  4. Remover atividade                                           │");
        println!("│  5. Visualizar grade semanal                                    │");
        println!("│  6. Ver estatísticas                                            │");
        println!("│  7. Buscar atividades                                           │");
        println!("│  8. Exportar para CSV                                           │");
        println!("│  9. Listar categorias                                           │");
        println!("│  0. Sair                                                        │");
        println!("└──────────────────────────────────────────────────────────────────┘");
    }
    
    fn get_user_input(&self, prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Erro ao ler entrada");
        input.trim().to_string()
    }
    
    fn add_activity_interactive(&mut self) {
        println!("\n=== ADICIONAR NOVA ATIVIDADE ===");
        
        let title = self.get_user_input("Título da atividade: ");
        if title.is_empty() {
            println!("Título não pode estar vazio!");
            return;
        }
        
        self.list_categories();
        let category = self.get_user_input("Categoria: ");
        
        println!("Dias disponíveis: Segunda, Terça, Quarta, Quinta, Sexta, Sábado, Domingo");
        let day = self.get_user_input("Dia da semana: ");
        
        let start_time = self.get_user_input("Horário de início (HH:MM): ");
        
        let duration_str = self.get_user_input("Duração em horas (ex: 0.5 para 30min, 1.5 para 1h30): ");
        let duration: f32 = match duration_str.parse() {
            Ok(d) => d,
            Err(_) => {
                println!("Duração inválida!");
                return;
            }
        };
        
        let location = self.get_user_input("Local (opcional): ");
        let location = if location.is_empty() { None } else { Some(location) };
        
        let description = self.get_user_input("Descrição (opcional): ");
        let description = if description.is_empty() { None } else { Some(description) };
        
        match self.organizer.add_activity(&title, &category, &day, &start_time, duration, location, description) {
            Ok(id) => println!("✅ Atividade criada com sucesso! ID: {}", id),
            Err(e) => println!("❌ Erro: {}", e),
        }
    }
    
    fn list_activities(&self) {
        println!("\n=== LISTA DE ATIVIDADES ===");
        let activities = self.organizer.get_all_activities();
        
        if activities.is_empty() {
            println!("Nenhuma atividade cadastrada.");
            return;
        }
        
        for activity in activities {
            println!("\n┌─────────────────────────────────────────────────────────────");
            println!("│ ID: {}", activity.id);
            println!("│ 📝 {}", activity.title);
            println!("│ 📅 {} às {}", activity.day, activity.start_time);
            println!("│ ⏱️  Duração: {}", WeeklyOrganizer::format_time(activity.duration));
            println!("│ 🏷️  Categoria: {}", self.organizer.categories.get(&activity.category).map_or(&activity.category, |c| &c.name));
            if let Some(location) = &activity.location {
                println!("│ 📍 Local: {}", location);
            }
            if let Some(description) = &activity.description {
                println!("│ 📄 Descrição: {}", description);
            }
            println!("└─────────────────────────────────────────────────────────────");
        }
    }
    
    fn edit_activity_interactive(&mut self) {
        println!("\n=== EDITAR ATIVIDADE ===");
        
        let id = self.get_user_input("ID da atividade para editar: ");
        
        // Verificar se atividade existe
        let activity = match self.organizer.activities.iter().find(|a| a.id == id) {
            Some(act) => act.clone(),
            None => {
                println!("Atividade não encontrada!");
                return;
            }
        };
        
        println!("Atividade atual: {}", activity.title);
        println!("Deixe em branco para manter o valor atual:");
        
        let title = self.get_user_input(&format!("Novo título ({}): ", activity.title));
        let title = if title.is_empty() { None } else { Some(title.as_str()) };
        
        let category = self.get_user_input(&format!("Nova categoria ({}): ", activity.category));
        let category = if category.is_empty() { None } else { Some(category.as_str()) };
        
        let day = self.get_user_input(&format!("Novo dia ({}): ", activity.day));
        let day = if day.is_empty() { None } else { Some(day.as_str()) };
        
        let start_time = self.get_user_input(&format!("Novo horário ({}): ", activity.start_time));
        let start_time = if start_time.is_empty() { None } else { Some(start_time.as_str()) };
        
        let duration_str =