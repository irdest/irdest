
const ASCII: &str = r#"      ,     .             
      (\,;,/)                    (\,/)
       (o o)\//,                 oo   '''//,        _
        \ /     \,             ,/_;~,        \,    / '
        `+'(  (   \    )       "'   \    (    \    !
           //  \   |_./              ',|  \    |__.'
         '~' '~----'                 '~  '~----''
                R A T    C O N T R O L
"#;
                       


#[async_std::main]
async fn main() {
    println!("{}", ASCII);

    
}
