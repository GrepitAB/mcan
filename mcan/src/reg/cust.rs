#[doc = "Register `CUST` reader"]
pub type R = crate::R<CUST_SPEC>;
#[doc = "Register `CUST` writer"]
pub type W = crate::W<CUST_SPEC>;
#[doc = "Customer Register\n\nYou can [`read`](crate::reg::generic::Reg::read) this register and get [`mrcfg::R`](R).  You can [`reset`](crate::reg::generic::Reg::reset), [`write`](crate::reg::generic::Reg::write), [`write_with_zero`](crate::reg::generic::Reg::write_with_zero) this register using [`mrcfg::W`](W). You can also [`modify`](crate::reg::generic::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CUST_SPEC;
impl crate::RegisterSpec for CUST_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcfg::R`](R) reader structure"]
impl crate::Readable for CUST_SPEC {}
#[doc = "`write(|w| ..)` method takes [`mrcfg::W`](W) writer structure"]
impl crate::Writable for CUST_SPEC {
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
