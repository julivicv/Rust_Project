use std::io::{Write, Read};

#[derive(Debug, Clone)]
pub struct Pedido {
    pub order_id: i64,
    pub user_id: i64,
    pub event_time: String,
    pub product_id: i64,
    pub price: f64,
}

impl Pedido {
    pub const TAMANHO_REGISTRO: usize = 62; // 8+8+30+8+8=62 (+1 para '\n' na gravação)

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::TAMANHO_REGISTRO);
        bytes.extend_from_slice(&self.order_id.to_le_bytes());
        bytes.extend_from_slice(&self.user_id.to_le_bytes());
        let t = format!("{:<30}", self.event_time);
        bytes.extend_from_slice(&t.as_bytes()[..30]);
        bytes.extend_from_slice(&self.product_id.to_le_bytes());
        bytes.extend_from_slice(&self.price.to_le_bytes());
        // não é obrigatório gravar '\n' no binário, grava só bytes fixos!
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let order_id = i64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let user_id = i64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let event_time = String::from_utf8_lossy(&bytes[16..46]).trim().to_string();
        let product_id = i64::from_le_bytes(bytes[46..54].try_into().unwrap());
        let price = f64::from_le_bytes(bytes[54..62].try_into().unwrap());
        Pedido { order_id, user_id, event_time, product_id, price }
    }
}

pub fn inserir_pedidos_ordenados(pedidos: &mut Vec<Pedido>, caminho: &str) -> std::io::Result<()> {
    pedidos.sort_by_key(|p| p.order_id);
    let mut arquivo = std::fs::File::create(caminho)?;
    for pedido in pedidos {
        let bytes = pedido.to_bytes();
        arquivo.write_all(&bytes)?;
    }
    Ok(())
}

pub fn mostrar_pedidos(caminho: &str, limite: usize) -> std::io::Result<Vec<Pedido>> {
    let mut arquivo = std::fs::File::open(caminho)?;
    let mut pedidos = Vec::new();
    let mut buffer = vec![0u8; Pedido::TAMANHO_REGISTRO];
    for _ in 0..limite {
        match arquivo.read_exact(&mut buffer) {
            Ok(_) => pedidos.push(Pedido::from_bytes(&buffer)),
            Err(_) => break,
        }
    }
    Ok(pedidos)
}