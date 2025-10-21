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
    let indice_produto_path = "indice_produtos.bin";
    let overflow_produto_path = "produtos_overflow.dat";

    loop {
        println!("\n=== MENU PRINCIPAL ===");
        println!("1 - Gerar arquivo binário de produtos a partir do CSV");
        println!("2 - Mostrar produtos (primeiros N)");
        println!("3 - Buscar produto por product_id (busca binária)");
        println!("4 - Construir índice parcial de produtos");
        println!("5 - Consultar produto via índice parcial");
        println!("6 - Consultar produto via índice (com debug)");
        println!("7 - Inserir novo produto");
        println!("8 - Remover produto por product_id");
        println!("9 - Mostrar estrutura do arquivo de índices");
        println!("10 - Reconstruir arquivo e índice");
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
                if !std::path::Path::new(produtos_path).exists() {
                    println!("Arquivo de produtos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o product_id para buscar:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                match buscar_produto_com_overflow(produtos_path, overflow_produto_path, chave).unwrap() {
                    Some(produto) => println!("Produto encontrado: {:?}", produto),
                    None => println!("Produto NÃO encontrado!"),
                }
            }
            "4" => {
                if !std::path::Path::new(produtos_path).exists() {
                    println!("Arquivo de produtos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe fator de esparsidade para índice parcial:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let fator = s.trim().parse().unwrap_or(10);
                let indice = construir_indice_parcial(produtos_path, fator).unwrap();
                indice.salvar_binario(indice_produto_path).unwrap();
                println!("Índice parcial construído e salvo em formato binário!");
            }
            "5" => {
                if !std::path::Path::new(produtos_path).exists() {
                    println!("Arquivo de produtos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o product_id para consulta via índice:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                let indice = IndiceParcial::carregar_binario(indice_produto_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                match consultar_com_indice_e_overflow(produtos_path, overflow_produto_path, &indice, chave).unwrap() {
                    Some(produto) => println!("Produto encontrado: {:?}", produto),
                    None => println!("Produto NÃO encontrado!"),
                }
            }
            "6" => {
                if !std::path::Path::new(produtos_path).exists() {
                    println!("Arquivo de produtos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o product_id para consulta via índice (com debug):");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                let indice = IndiceParcial::carregar_binario(indice_produto_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                match consultar_com_indice_e_overflow_debug(produtos_path, overflow_produto_path, &indice, chave).unwrap() {
                    Some(produto) => println!("\n✅ Produto encontrado: {:?}", produto),
                    None => println!("\n❌ Produto NÃO encontrado!"),
                }
            }
            "7" => {
                println!("Informe dados do novo produto:");
                let product_id = read_num("product_id");
                let category_alias = read_string("category_alias");
                let price = read_float("price");
                let material = read_string("material");
                let stone = read_string("stone");

                let mut indice = IndiceParcial::carregar_binario(indice_produto_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
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
            "8" => {
                if !std::path::Path::new(produtos_path).exists() {
                    println!("Arquivo de produtos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o product_id para remoção:");
                let chave = read_num("product_id");
                if remover_produto_com_overflow(produtos_path, overflow_produto_path, chave).unwrap() {
                    println!("Produto removido!");
                } else {
                    println!("Produto NÃO encontrado para remoção!");
                }
            }
            "9" => {
                mostrar_estrutura_indices(indice_produto_path);
            }
            "10" => {
                println!("Reconstruindo arquivo e índice...");
                let mut indice = IndiceParcial::carregar_binario(indice_produto_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                reconstruir_arquivo_e_indice(produtos_path, overflow_produto_path, &mut indice).unwrap();
                indice.salvar_binario(indice_produto_path).unwrap();
                println!("✅ Reconstrução concluída!");
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
    let indice_pedido_path = "indice_pedidos.bin";
    let overflow_pedido_path = "pedidos_overflow.dat";

    loop {
        println!("\n=== MENU PEDIDOS ===");
        println!("1 - Gerar arquivo binário de pedidos a partir do CSV");
        println!("2 - Mostrar pedidos (primeiros N)");
        println!("3 - Buscar pedido por order_id (busca binária)");
        println!("4 - Construir índice parcial de pedidos");
        println!("5 - Consultar pedido via índice parcial");
        println!("6 - Consultar pedido via índice (com debug)");
        println!("7 - Inserir novo pedido");
        println!("8 - Remover pedido por order_id");
        println!("9 - Mostrar estrutura do arquivo de índices");
        println!("10 - Reconstruir arquivo e índice");
        println!("0 - Voltar");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut escolha = String::new();
        io::stdin().read_line(&mut escolha).unwrap();
        match escolha.trim() {
            "1" => {
                println!("Gerando arquivo binário de pedidos a partir do CSV...");
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
                println!("Arquivo de pedidos criado e ordenado!");
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
            "3" => {
                if !std::path::Path::new(pedidos_path).exists() {
                    println!("Arquivo de pedidos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o order_id para buscar:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                match busca_binaria_arquivo_pedido(pedidos_path, chave).unwrap() {
                    Some(pedido) => println!("Pedido encontrado: {:?}", pedido),
                    None => println!("Pedido NÃO encontrado!"),
                }
            }
            "4" => {
                if !std::path::Path::new(pedidos_path).exists() {
                    println!("Arquivo de pedidos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe fator de esparsidade para índice parcial:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let fator = s.trim().parse().unwrap_or(10);
                let indice = construir_indice_parcial(pedidos_path, fator).unwrap();
                indice.salvar_binario(indice_pedido_path).unwrap();
                println!("Índice parcial construído e salvo em formato binário!");
            }
            "5" => {
                if !std::path::Path::new(pedidos_path).exists() {
                    println!("Arquivo de pedidos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o order_id para consulta via índice:");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                let indice = IndiceParcial::carregar_binario(indice_pedido_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                match consultar_com_indice_pedido(pedidos_path, &indice, chave).unwrap() {
                    Some(pedido) => println!("Pedido encontrado: {:?}", pedido),
                    None => println!("Pedido NÃO encontrado!"),
                }
            }
            "6" => {
                if !std::path::Path::new(pedidos_path).exists() {
                    println!("Arquivo de pedidos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o order_id para consulta via índice (com debug):");
                let mut s = String::new();
                io::stdin().read_line(&mut s).unwrap();
                let chave = s.trim().parse().unwrap_or(0);
                let indice = IndiceParcial::carregar_binario(indice_pedido_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                match consultar_com_indice_pedido_debug(pedidos_path, &indice, chave).unwrap() {
                    Some(pedido) => println!("\n✅ Pedido encontrado: {:?}", pedido),
                    None => println!("\n❌ Pedido NÃO encontrado!"),
                }
            }
            "7" => {
                println!("Informe dados do novo pedido:");
                let order_id = read_num("order_id");
                let user_id = read_num("user_id");
                let event_time = read_string("event_time");
                let product_id = read_num("product_id");
                let price = read_float("price");

                let mut indice = IndiceParcial::carregar_binario(indice_pedido_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                let pedido = Pedido {
                    order_id,
                    user_id,
                    event_time,
                    product_id,
                    price,
                };
                inserir_novo_pedido(pedidos_path, overflow_pedido_path, pedido, &mut indice).unwrap();
                indice.salvar_binario(indice_pedido_path).unwrap();
                println!("Novo pedido inserido (área de overflow)!");
            }
            "8" => {
                if !std::path::Path::new(pedidos_path).exists() {
                    println!("Arquivo de pedidos nao encontrado! Execute primeiro a opcao 1.");
                    continue;
                }
                println!("Informe o order_id para remoção:");
                let chave = read_num("order_id");
                if remover_pedido_com_overflow(pedidos_path, overflow_pedido_path, chave).unwrap() {
                    println!("Pedido removido!");
                } else {
                    println!("Pedido NÃO encontrado para remoção!");
                }
            }
            "9" => {
                mostrar_estrutura_indices(indice_pedido_path);
            }
            "10" => {
                println!("Reconstruindo arquivo e índice...");
                let mut indice = IndiceParcial::carregar_binario(indice_pedido_path).unwrap_or_else(|_| {
                    println!("Indice nao encontrado, criando novo com fator 10");
                    IndiceParcial::novo(10)
                });
                reconstruir_arquivo_e_indice_pedido(pedidos_path, overflow_pedido_path, &mut indice).unwrap();
                indice.salvar_binario(indice_pedido_path).unwrap();
                println!("✅ Reconstrução concluída!");
            }
            "0" => {
                println!("Voltando...");
                break;
            }
            _ => println!("Opção inválida!"),
        }
    }
}

fn mostrar_estrutura_indices(indice_path: &str) {
    println!("\n=== ESTRUTURA DO ARQUIVO DE INDICES ===");
    
    if !std::path::Path::new(indice_path).exists() {
        println!("Arquivo de indice nao encontrado: {}", indice_path);
        println!("Dica: Execute primeiro a opcao 4 para construir o indice parcial");
        return;
    }
    
    match IndiceParcial::carregar_binario(indice_path) {
        Ok(indice) => {
            println!("Indice carregado com sucesso!");
            println!("Fator de esparsidade: {}", indice.fator_esparsidade);
            println!("Total de entradas no indice: {}", indice.entradas.len());
            println!();
            
            if indice.entradas.is_empty() {
                println!("O indice esta vazio!");
                return;
            }
            
            let primeira_chave = indice.entradas[0].chave;
            let ultima_chave = indice.entradas[indice.entradas.len() - 1].chave;
            println!("Primeira chave: {}", primeira_chave);
            println!("Ultima chave: {}", ultima_chave);
            println!("Intervalo de chaves: {} a {}", primeira_chave, ultima_chave);
            println!();
            
            println!("Primeiras 10 entradas do indice:");
            println!("{:<8} {:<12} {:<15}", "Pos", "Chave", "Posicao Arquivo");
            println!("{}", "-".repeat(40));
            
            for (i, entrada) in indice.entradas.iter().take(10).enumerate() {
                println!("{:<8} {:<12} {:<15}", 
                    i, 
                    entrada.chave, 
                    entrada.posicao
                );
            }
            
            if indice.entradas.len() > 10 {
                println!("... e mais {} entradas", indice.entradas.len() - 10);
            }
            println!();
            
            if let Ok(metadata) = std::fs::metadata(indice_path) {
                println!("Informacoes do arquivo:");
                println!("   Tamanho: {} bytes", metadata.len());
                println!("   Caminho: {}", indice_path);
                println!("   Formato: Binario");
                
                let tamanho_cabecalho = 8;
                let tamanho_entradas = indice.entradas.len() * 16;
                let tamanho_esperado = tamanho_cabecalho + tamanho_entradas;
                println!("   Tamanho esperado: {} bytes", tamanho_esperado);
                println!("   Tamanho por entrada: 16 bytes (8 bytes chave + 8 bytes posicao)");
            }
            
            if indice.entradas.len() > 1 {
                let mut intervalos = Vec::new();
                for i in 1..indice.entradas.len() {
                    let intervalo = indice.entradas[i].chave - indice.entradas[i-1].chave;
                    intervalos.push(intervalo);
                }
                
                if !intervalos.is_empty() {
                    let media_intervalo = intervalos.iter().sum::<i64>() as f64 / intervalos.len() as f64;
                    let intervalo_min = *intervalos.iter().min().unwrap();
                    let intervalo_max = *intervalos.iter().max().unwrap();
                    
                    println!();
                    println!("Estatisticas de distribuicao:");
                    println!("   Intervalo medio entre chaves: {:.2}", media_intervalo);
                    println!("   Menor intervalo: {}", intervalo_min);
                    println!("   Maior intervalo: {}", intervalo_max);
                }
            }
            
            println!();
            println!("Como funciona o indice:");
            println!("   - Cada entrada aponta para uma posicao no arquivo de produtos");
            println!("   - O fator de esparsidade {} significa que a cada {} produtos, uma entrada e criada", 
                     indice.fator_esparsidade, indice.fator_esparsidade);
            println!("   - Para buscar um produto, o sistema usa busca binaria no indice");
            println!("   - Depois busca sequencialmente no intervalo indicado pelo indice");
        }
        Err(e) => {
            println!("Erro ao carregar o indice: {}", e);
            println!("Verifique se o arquivo existe e esta no formato correto");
        }
    }
}

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
