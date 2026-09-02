1) Baixe o arquivo ponte_installer.exe nos dois computadores entre os quais você deseja transferir arquivos
2) Execute como administrador
3) O programa será instalado e o desinstalador será criado
4) Abra o terminal e digite ponte, os comandos devem ser exibidos confirmando que o programa foi instalado corretamente

Requisitos, os dois computadores devem estar em LAN, não é necessário conexão com a internet!

Transferindo arquivos:
Para receber:
Crie uma pasta qualquer no PC que deseja receber os arquivos, abra o terminal nesta pasta e digite: "ponte receber --auto"
Uma mensagem deve aparecer indicando o ip que o servidor está escutando, mantenha este terminal aberto com a escuta

Para enviar:
Crie uma pasta qualquer no PC que vai enviar os arquivos, abra o terminal nela e digite: "ponte enviar"
Uma nova pasta deve ser criada com o nome de "enviar", coloque dentro dela os arquivos a serem enviados
Digite: "ponte enviar" novamente, o programa vai pedir o ip do servidor, o mesmo que o servidor informou antes na escuta, digite ele corretamente e o envio será feito na hora


Novo recurso: não é necessário digitar o ip completo nas versões mais recentes como v1.3, se sua máscara de rede for 255.255.255.0, basta digitar o final do ip, a outra parte será preenchida automaticamente
Exemplo, se a máscara for 255.255.255.0 e você quer digitar o ip 192.168.0.108, basta digitar "108"
se a máscara for 255.255.0.0 basta digitar os final também como "140.108", o resto é preenchido automaticamente
