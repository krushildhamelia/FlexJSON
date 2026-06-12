pub mod error;
pub mod extractor;
pub mod lexer;
pub mod normalizer;
pub mod parser;
pub mod types;

use error::{ParseError, ParseWarning};
use serde::Serialize;
use types::{Fragment, ParseNode};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub fragments: Vec<NormalizedFragment>,
    pub warnings: Vec<ParseWarning>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedFragment {
    pub index: usize,
    pub source: Fragment,
    pub tree: ParseNode,
    pub json: String,
}

pub fn parse(input: &str) -> ParseResult {
    let mut fragments = Vec::new();
    let mut errors = Vec::new();

    let extracted = extractor::extract(input);

    for (i, ext) in extracted.into_iter().enumerate() {
        let mut parser = parser::Parser::new(&ext.raw);
        match parser.parse() {
            Ok(tree) => {
                let json = normalizer::normalize(&tree);
                fragments.push(NormalizedFragment {
                    index: i,
                    source: ext,
                    tree,
                    json,
                });
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    ParseResult {
        fragments,
        warnings: vec![],
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_json_round_trip() {
        let input = r#"{"a": 1, "b": [true, null, "test"]}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.fragments.len(), 1);
        let expected = r#"{"a":1,"b":[true,null,"test"]}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_mixed_separators() {
        let input = r#"{a: 1, b = 2, c => 3, d -> 4}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"a":1,"b":2,"c":3,"d":4}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_mixed_quoting() {
        let input = r#"{ "a": 1, 'b': 2, c: 3 }"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"a":1,"b":2,"c":3}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_unquoted_boundary_heuristic() {
        let input = r#"{name: FlexJSON "Neon, test: a}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"name":"FlexJSON \"Neon","test":"a"}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_embedded_commas() {
        let input = r#"{x: a,b,c, y: z}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        let expected = r#"{"x":"a,b,c","y":"z"}"#;
        assert_eq!(result.fragments[0].json, expected);
    }

    #[test]
    fn test_multi_fragment() {
        let input = "Here is some text {a: 1} and another [2, 3]";
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.fragments.len(), 2);
        assert_eq!(result.fragments[0].json, r#"{"a":1}"#);
        assert_eq!(result.fragments[1].json, r#"[2,3]"#);
    }

    #[test]
    fn test_problematic_log_parsing() {
        let input = r#"bookie-1-a.log.4.gz:[2026-06-09 14:53:09,372Z][INFO][Actor-Executor-3A38525-1A23060:2258][client.rfq.RFQRequestQuoteFromCommandProcessor] Updating book {id=d0756546-d245-5a91-9a9b-9722f3da91dd, activeDealers=[1B95857, 1A75193, 100194, 1A08567], allocations={}, anonymousDealerAllowed=false, assetClass=FixedIncome, autoEx=false, autoExAutoPassing=false, autoExecuted=false, autoexSecondsEvaluation=0, bookType=BookType:REQUEST_FOR_QUOTE, branch=3A38525, dealerBenchmarkAllowed=false, dealerDiscovery=false, dealType=DealType:Outright, handlingType=HandlingType:RFQ, inventory=false, login=Egriffin, notTradedWithBestQuote=false, numberOfCompetitors=74, onlyShowBestSTM=false, orderBookData={}, portfolioId=20260609-145247347_8000, portfolioRequestId=26060914-5308-4ca5-b4d1-6d2169671253, quoteInquiryType=BIN, requestedDealers=[1B95857, 1A75193, 100194, 1A08567], requestedGoodForSeconds=600, requestedSide=QuoteSide:TWO_SIDED, requestedValidSeconds=300, requestedValueType=ValueType:PRICE, requestId=0e2b2525-9007-413c-88b0-12740f18a5f7, requestTime=2026-06-09T10:53:08.854-04:00, requestType=SINGLE_PLATFORM_MULTI_DIRECTDEALER_RFQ, RFQExpiryTime=2026-06-09T10:58:08.854-04:00, routingKey=3A38525-1A23060, STMSourceFilter=[]} from request {id=26060914-5308-4ca5-b4d1-6d2169671253, allocations={}, allocationToBeTakenFromRequest=false, allowDealerBenchmark=true, anonymous=false, anonymousDealerAllowed=true, AONRequest=false, assetClass=FixedIncome, autoexecutionEnabled=false, bookVersion=0, dealerDiscovery=false, dealType=DealType:Outright, directPublishers=[100194, 1B95857, 1A08567], excludedExecutingBrokers=[], expireTime=2026-06-09T23:59:00.000-04:00, federationId=1A23060, goodForSeconds=600, handlingType=HandlingType:RFQ, includeSTMs=false, inventory=false, isAckRequired=false, isAutoRouted=false, isGrouped=false, isGroupParent=false, isLocked=false, isStagedByFIX=false, isWaveOrder=false, onlyShowBestSTM=false, platformId=1A23060, platformProtocol=ValueType:PRICE, portfolioId=20260609-145247347_8000, portfolioOrders=[{allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=SIRI 4.125 07/01/2030 144A, instrumentCodeType=TSproductId, instrumentId=2544154801, orderId=20260608-3B16717-000000036-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=1, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309087-209-128, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=BBWI 6.750 07/01/2036, instrumentCodeType=TSproductId, instrumentId=2199896049, orderId=20260608-3B16717-000000042-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=2, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309089-684-129, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=SABHLD 11.125 06/15/2029 144A, instrumentCodeType=TSproductId, instrumentId=3233523370, orderId=20260608-3B16717-000000050-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=3, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309090-618-130, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=JFSABZ 8.500 12/01/2032, instrumentCodeType=TSproductId, instrumentId=3305671176, orderId=20260608-3B16717-000000047-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=100000.00000000, sequenceNumber=4, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309092-733-131, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=CHTR 4.250 02/01/2031 144A, instrumentCodeType=TSproductId, instrumentId=2551706113, orderId=20260608-3B16717-000000038-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=100000.00000000, sequenceNumber=5, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309093-565-132, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=SESGFP 5.300 04/04/2043 144A, instrumentCodeType=TSproductId, instrumentId=2121464614, orderId=20260608-3B16717-000000051-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=25000.00000000, sequenceNumber=6, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309094-643-133, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=CHHCF 11.750 09/01/2030 144A, instrumentCodeType=TSproductId, instrumentId=3184281520, orderId=20260608-3B16717-000000046-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=25000.00000000, sequenceNumber=7, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309095-639-134, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=OMF 5.375 11/15/2029, instrumentCodeType=TSproductId, instrumentId=2485105423, orderId=20260608-3B16717-000000053-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=8, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309096-111-135, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=HAHGRO 9.750 10/01/2031 144A, instrumentCodeType=TSproductId, instrumentId=3057297070, orderId=20260608-3B16717-000000048-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=9, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309097-959-136, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=BCECN 7.000 09/15/2055 FRN, instrumentCodeType=TSproductId, instrumentId=3113231300, orderId=20260608-3B16717-000000043-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=10, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309098-913-137, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=CNC 2.500 03/01/2031, instrumentCodeType=TSproductId, instrumentId=2614822336, orderId=20260608-3B16717-000000035-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=11, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309099-130-138, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=NRG 6.000 01/15/2036 144A, instrumentCodeType=TSproductId, instrumentId=3206328580, orderId=20260608-3B16717-000000052-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=50000.00000000, sequenceNumber=12, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309100-443-139, venueId=OTC}, {allocations={}, allowedBrokersList=[1A40714, 100061, BNP, 1A32136, 1A29247, MIZUHO, 1B76084, 1B76086, NOMURA, 101627, 102838, 101662, 1A79852, 100613, 1B45602, 1B97806, 1B21339, 102372, 100194, 1A85611, 1B90577, MS, 1A87716, 100792, 1A90448, 1B40632, 1B90131, 102924, 1B09705, 1B83933, 1B06488, 100069, 1A00848, 101710, BARCLAYS, 101755, 102524, 100546, 1B23802, 1A82653, 101691, 1B63741, 1A08567, 3A13263, 1A60462, 101290, 1B95692, 1A46796, 1A73777, 1A45266, 1B84796, 1B77625, 102610, 1B11893, 1B44075, 101688, 100054, 1A56810, 101661, UBS, 1A95676, 1A27329, 100166, 1A39542, 102784, 101697, 101533, 1A42867, 1B35316, DB], allowedDirectBrokersList=[1B95857, 100194, 1A08567], anonymousDealerAllowed=false, autoEx=false, currency=USD, description=FMC 8.450 11/01/2055 FRN, instrumentCodeType=TSproductId, instrumentId=3154135610, orderId=20260608-3B16717-000000041-STG, platformProtocol=ValueType:PRICE, priceBasis=PercentPar, priceType=Market, priceValueType=ValueType:PRICE, quantity=25000.00000000, sequenceNumber=13, settlementDate=2026-06-10, settlementDateProvided=true, side=QuoteSide:BUY, slicedOrderId=20260609-145309101-848-140, venueId=OTC}], publishers=[100194, 1B95857, 1A08567], quoteInquiryType=BIN, requester=3A38525, requestId=0e2b2525-9007-413c-88b0-12740f18a5f7, requestTime=2026-06-09T10:53:08.854-04:00, requestType=SINGLE_PLATFORM_MULTI_DIRECTDEALER_RFQ, sendTSFIBenchmarksToDealers=false, settlementDateProvided=false, side=QuoteSideTWO_SIDED, sliceOrder=true, spotType=OneStep, standardMode=true, user=Egriffin, validSeconds=300, valueType=ValueType:PRICE}"#;
        let result = parse(input);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.fragments.len(), 6);
        assert_eq!(result.fragments[0].json.is_empty(), false);
        assert_eq!(result.fragments[1].json.is_empty(), false);
    }
}
