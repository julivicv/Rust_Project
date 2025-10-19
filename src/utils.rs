// Funções auxiliares, por exemplo, para reconstrução, gerenciamento de overflow etc.
use crate::produto::Produto;
use crate::indice::IndiceParcial;
use std::fs::{File, OpenOptions};
use std::io::Write;

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
    let todos_produtos: Vec<Produto> = Vec::new();
    // Ler principal e overflow e unir registros válidos,
    // ordenar, reescrever e reconstruir índice:
    // inserir_produtos_ordenados(&mut todos_produtos, caminho_principal)?;
    // *indice = construir_indice_parcial(caminho_principal, indice.fator_esparsidade)?;
    std::fs::write(caminho_overflow, "")?;
    Ok(())
}

// Demais utilitários podem ser adicionados aqui.
