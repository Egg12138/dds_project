local helper = {}
spi = require("spi")

-- TODO remove some parameters
function helper.setup(
  id,cs,CPHA,CPOL,dataw,bandrate,bitdict,ms,mode
)
  spi.setup(id,cs,CPHA,CPOL,dataw,bandrate,bitdict,ms,mode)

end



return helper