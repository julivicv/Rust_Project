mod produto;
mod indice;
mod utils;
mod pedido;

use std::io::{self, Write};
use produto::*;
use indice::*;
use utils::*;
use pedido::*;


const CSV_PATH: &str = "jewelry.csv";

fn main() {
    loop {
        println!("\n=== MENU PRINCIPAL ===");
        println!("1 - Funções de Produtos");
        println!("2 - Funções de Pedidos");
        println!("0 - Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();


        let mut escolha = String::new();
        io::stdin().read_line(&mut escolha).unwrap();
        match escolha.trim() {
            "1" => menu_produtos(),
            "2" => menu_pedidos(),
            "0" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opção inválida!"),
        }
    }
}

fn menu_produtos() {
    let produtos_path = "produtos.dat";
    let pedidos_path = "pedidos.dat";
    let indice_produto_path = "indice_produtos.json";
    let overflow_produto_path = "produtos_overflow.dat";

    loop {
        println!("\n=== MENU PRINCIPAL ===");
        println!("1 - Gerar arquivo binário de produtos a partir do CSV");
        println!("2 - Mostrar produtos (primeiros N)");
        println!("3 - Buscar produto por product_id (busca binária)");
        println!("4 - Construir índice parcial de produtos");
        println!("5 - Consultar produto via índice parcial");
        println!("6 - Inserir novo produto");
        println!("7 - Remover produto por product_id");
        println!("0 - Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut escolha = String::new();
        io::stdin().read_line(&mut escolha).unwrap();
        match escolha.trim() {
            "1" => {
                println!("Gerando arquivo binário de produtos a partir do CSV...");
                let mut produtos: Vec<Produto> = Vec::new();
                let mut rdr = csv::Reader::from_path(CSV_PATH).unwrap();
                for result in rdr.records() {
                    let record = match result {
                        Ok(rec) => rec,
                        Err(_) => continue, // pula linha inválida
                    };
                    if record.len() < 13 { continue; } // ignora registros incompletos
                
                    let product_id = record[2].parse::<i64>().unwrap_or(0);
                    let category_alias = record[5].to_string();
                    let price = record[7].parse::<f64>().unwrap_or(0.0);
                    let material = record[11].to_string();
                    let stone = record[12].to_string();
                    produtos.push(Produto {
                        product_id,
                        category_alias,
                        price,
                        material,
                        stone,
                    });
                }
                inserir_produtos_ordenados(&mut produtos, produtos_path).unwrap();
                println!("Arquivo de produtos criado e ordenado!");
            }
            "2" => {
                println!("Quantos produtos deseja mostrar?");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let n = s.trim().parse().unwrap_or(10);
                let lista = mostrar_produtos(produtos_path, n).unwrap();
                for p in lista {
                    println!("{:?}", p);
                }
            }
            "3" => {
                println!("Informe o product_id para buscar:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                match busca_binaria_arquivo(produtos_path, chave).unwrap() {
                    Some(produto) => println!("Produto encontrado: {:?}", produto),
                    None => println!("Produto NÃO encontrado!"),
                }
            }
            "4" => {
                println!("Informe fator de esparsidade para índice parcial:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let fator = s.trim().parse().unwrap_or(10);
                let indice = construir_indice_parcial(produtos_path, fator).unwrap();
                indice.salvar(indice_produto_path).unwrap();
                println!("Índice parcial construído e salvo!");
            }
            "5" => {
                println!("Informe o product_id para consulta via índice:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                let indice = IndiceParcial::carregar(indice_produto_path, 10).unwrap();
                match consultar_com_indice(produtos_path, &indice, chave).unwrap() {
                    Some(produto) => println!("Produto encontrado: {:?}", produto),
                    None => println!("Produto NÃO encontrado no intervalo do índice!"),
                }
            }
            "6" => {
                println!("Informe dados do novo produto:");
                let product_id = read_num("product_id");
                let category_alias = read_string("category_alias");
                let price = read_float("price");
                let material = read_string("material");
                let stone = read_string("stone");

                let mut indice = IndiceParcial::carregar(indice_produto_path, 10).unwrap();
                let produto = Produto {
                    product_id,
                    category_alias,
                    price,
                    material,
                    stone,
                };
                inserir_novo_produto(produtos_path, overflow_produto_path, produto, &mut indice).unwrap();
                println!("Novo produto inserido (área de overflow)!");
            }
            "7" => {
                println!("Informe o product_id para remoção:");
                let chave = read_num("product_id");
                if remover_produto(produtos_path, chave).unwrap() {
                    println!("Produto removido!");
                } else {
                    println!("Produto NÃO encontrado para remoção!");
                }
            }
            "0" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opção inválida!"),
        }
    }
}
fn menu_pedidos() {
    let pedidos_path = "pedidos.dat";
    loop {
        println!("\n===== MENU PEDIDOS =====");
        println!("1 - Gerar arquivo de pedidos a partir do CSV");
        println!("2 - Mostrar pedidos (primeiros N)");
        println!("0 - Voltar");
        print!("Escolha: ");
        io::stdout().flush().unwrap();

        let mut esc = String::new();
        io::stdin().read_line(&mut esc).unwrap();

        match esc.trim() {
            "1" => {
                println!("Gerando arquivo de pedidos binário do CSV...");
                let mut pedidos: Vec<Pedido> = Vec::new();
                let mut rdr = csv::Reader::from_path(CSV_PATH).unwrap();
                for result in rdr.records() {
                    let record = match result { Ok(rec) => rec, Err(_) => continue };
                    if record.len() < 13 { continue; }
                    let order_id = record[1].parse::<i64>().unwrap_or(0);
                    let user_id = record[8].parse::<i64>().unwrap_or(0);
                    let event_time = record[0].to_string();
                    let product_id = record[2].parse::<i64>().unwrap_or(0);
                    let price = record[7].parse::<f64>().unwrap_or(0.0);
                    pedidos.push(Pedido { order_id, user_id, event_time, product_id, price });
                }
                inserir_pedidos_ordenados(&mut pedidos, pedidos_path).unwrap();
                println!("Arquivo de pedidos criado!");
            }
            "2" => {
                println!("Quantos pedidos deseja mostrar?");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let n = s.trim().parse().unwrap_or(10);
                let lista = mostrar_pedidos(pedidos_path, n).unwrap();
                for p in lista {
                    println!("{:?}", p);
                }
            }
            "0" => break,
            _ => println!("Opção inválida!"),
        }
    }
}

// Funções auxiliares para ler dados do menu
fn read_num(msg: &str) -> i64 {
    print!("{}: ", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().parse().unwrap_or(0)
}
fn read_float(msg: &str) -> f64 {
    print!("{}: ", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().parse().unwrap_or(0.0)
}
fn read_string(msg: &str) -> String {
    print!("{}: ", msg);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}
