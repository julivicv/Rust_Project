use crate::produto::Produto;
use crate::indice::{IndiceParcial, construir_indice_parcial};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

pub fn inserir_novo_produto(caminho_arquivo: &str, caminho_overflow: &str, produto: Produto, indice: &mut IndiceParcial) -> std::io::Result<()> {
    let mut arquivo_overflow = OpenOptions::new().create(true).append(true).open(caminho_overflow)?;
    let bytes = produto.to_bytes();
    arquivo_overflow.write_all(&bytes)?;
    let tam_principal = File::open(caminho_arquivo)?.metadata()?.len();
    let tam_overflow = arquivo_overflow.metadata()?.len();
    if tam_overflow as f64 > tam_principal as f64 * 0.1 {
        reconstruir_arquivo_e_indice(caminho_arquivo, caminho_overflow, indice)?;
    }
    Ok(())
}

pub fn reconstruir_arquivo_e_indice(
    caminho_principal: &str, 
    caminho_overflow: &str, 
    indice: &mut IndiceParcial
) -> std::io::Result<()> {
    println!("Iniciando reconstrucao do arquivo e indice...");
    
    let mut todos_produtos = Vec::new();
    
    println!("Lendo produtos do arquivo principal...");
    if let Ok(mut arquivo_principal) = File::open(caminho_principal) {
        let tamanho = arquivo_principal.metadata()?.len();
        let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
        let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
        
        for i in 0..num_registros {
            let pos = i * Produto::TAMANHO_REGISTRO as u64;
            arquivo_principal.seek(SeekFrom::Start(pos))?;
            match arquivo_principal.read_exact(&mut buffer) {
                Ok(_) => {
                    let produto = Produto::from_bytes(&buffer);
                    if produto.product_id != -1 {
                        todos_produtos.push(produto);
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    println!("Lendo produtos do arquivo de overflow...");
    if let Ok(mut arquivo_overflow) = File::open(caminho_overflow) {
        let tamanho = arquivo_overflow.metadata()?.len();
        let num_registros = tamanho / Produto::TAMANHO_REGISTRO as u64;
        let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
        
        for i in 0..num_registros {
            let pos = i * Produto::TAMANHO_REGISTRO as u64;
            arquivo_overflow.seek(SeekFrom::Start(pos))?;
            match arquivo_overflow.read_exact(&mut buffer) {
                Ok(_) => {
                    let produto = Produto::from_bytes(&buffer);
                    if produto.product_id != -1 {
                        todos_produtos.push(produto);
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    println!("Total de produtos validos encontrados: {}", todos_produtos.len());
    
    println!("Ordenando produtos por product_id...");
    todos_produtos.sort_by_key(|p| p.product_id);
    
    println!("Reescrevendo arquivo principal...");
    let mut arquivo_principal = File::create(caminho_principal)?;
    for produto in &todos_produtos {
        let bytes = produto.to_bytes();
        arquivo_principal.write_all(&bytes)?;
    }
    
    println!("Limpando arquivo de overflow...");
    std::fs::write(caminho_overflow, "")?;
    
    println!("Reconstruindo indice...");
    *indice = construir_indice_parcial(caminho_principal, indice.fator_esparsidade)?;
    
    let indice_path = "indice_produtos.bin";
    indice.salvar_binario(indice_path)?;
    
    println!("Reconstrucao concluida!");
    println!("   Produtos no arquivo principal: {}", todos_produtos.len());
    println!("   Entradas no indice: {}", indice.entradas.len());
    
    Ok(())
}

