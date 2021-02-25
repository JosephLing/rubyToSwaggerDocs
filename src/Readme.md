# Goal:
Create a parser for a ruby on rails project to be able to generate a typescript defentions file for the all endpoints of the API.
In doing so be able to document the API fully as MCV will be modelled in parsing (model will be a bit harder).

# Parsing TODO:
- jbuilder and jb parser for views and linking views to the controller
    handle custom view rendering:
    - respond_to 
    - template.render
    - check: is `schema.rb` a reliable source of truth to parse? as that will give us a good mapping for instance varaibles to use in the views
- concerns and importing - i.e getting scoping working properly.... 
    - concern parsing...
        - configuration actions handling
        - error return statements that don't use a view e.g. json()
    - scoping
        - class super and self
        - plan of what do we do about models
        - instance varaibles
    - object inheritance so that we can understand what logic is overlapped on each other as such
    - method handling (scope: local, object inheritance, other??? - need to seriously consider other) [HARD ONE!!!]
        - params and instance varaibles 

## checking/linting
- tracking instance varaibles and then mapping them to schema (either custom gem or parsing schema generated file to get info) 
    making the properties match up
- (potentail) detecting code changes - used for reporting
- (done) does routes file match controllers, although this could already be a feature I would imagine in rails 

## Typescript
- After we have all info we just need to convert into TS and then we can use a crate to format the code for us.
- Another alternative is to create swagger spec from the data (
https://lib.rs/crates/struct2swagger) and then use https://github.com/acacode/swagger-typescript-api (if bundling is an issue we might be able to wasm it up into a js plugin.... but the performance woudln't be as good...)
# Done
- parsing routes file (test.routes generated by doing `bundle exec rails routes > test.routes`)
- controller and module parsing
    - params and beginnings of instance varaibles, config/actions
    - requires parsed properly (although just ignored atm)
- mapping between controller.method and requests 

# Future goals
- parse routes.rb literally in rust


# rough plan notes:
- ast
- nodes kinda
    - routes - custom file
    - controllers - folder
        - concerns - in controllers
    - views
    - schema.rb
    - helpers/others - list of folders with files

`rts controllers/ views/ routes --other helpers`

1. basic parse
2. scope  - rerun knowing what everything is as such
3. controller mapped to views using models
4. match to routes