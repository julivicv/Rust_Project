use serde::{Serialize, Deserialize};
use std::io::{Read, Write};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndiceEntry {
    pub chave: i64,
    pub posicao: u64,
}

#[derive(Debug, Clone)]
pub struct IndiceParcial {
    pub entradas: Vec<IndiceEntry>,
    pub fator_esparsidade: usize,
}

impl IndiceParcial {
    pub fn novo(fator_esparsidade: usize) -> Self {
        IndiceParcial {
            entradas: Vec::new(),
            fator_esparsidade,
        }
    }

    pub fn adicionar_entrada(&mut self, chave: i64, posicao: u64) {
        self.entradas.push(IndiceEntry { chave, posicao });
    }

    pub fn salvar(&self, caminho: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(&self.entradas)?;
        std::fs::write(caminho, json)?;
        Ok(())
    }

    pub fn carregar(caminho: &str, fator: usize) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(caminho)?;
        let entradas: Vec<IndiceEntry> = serde_json::from_str(&json)?;
        Ok(IndiceParcial {
            entradas,
            fator_esparsidade: fator,
        })
    }

    pub fn buscar_posicao(&self, chave: i64) -> Option<(usize, u64)> {
        let mut esq = 0;
        let mut dir = self.entradas.len();
        while esq < dir {
            let meio = (esq + dir) / 2;
            if self.entradas[meio].chave < chave {
                esq = meio + 1;
            } else if self.entradas[meio].chave > chave {
                dir = meio;
            } else {
                return Some((meio, self.entradas[meio].posicao));
            }
        }
        if esq > 0 {
            Some((esq - 1, self.entradas[esq - 1].posicao))
        } else {
            Some((0, 0))
        }
    }
}

pub fn construir_indice_parcial(caminho_arquivo: &str, fator: usize) 
    -> std::io::Result<IndiceParcial> 
{
    use crate::produto::Produto;
    let mut indice = IndiceParcial::novo(fator);
    let mut arquivo = std::fs::File::open(caminho_arquivo)?;
    let mut buffer = vec![0u8; Produto::TAMANHO_REGISTRO];
    let mut contador = 0;
    let mut posicao = 0u64;
    loop {
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => {
                if contador % fator == 0 {
                    let produto = Produto::from_bytes(&buffer);
                    indice.adicionar_entrada(produto.product_id, posicao);
                }
                contador += 1;
                posicao += Produto::TAMANHO_REGISTRO as u64;
            }
            Err(_) => break,
        }
    }
    Ok(indice)
}
